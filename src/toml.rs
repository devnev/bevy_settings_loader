use std::any::TypeId;
use std::ops::Deref;
use std::str::from_utf8;

use bevy::asset::{AssetLoader, AsyncReadExt, LoadedUntypedAsset};
use bevy::prelude::*;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use thiserror::Error;

use crate::plugin::SettingsPluginSettings;

pub struct TomlAssetPlugin;

impl Plugin for TomlAssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(TomlAssetLoader)
            .init_asset::<TomlAsset>();
    }
}

#[derive(Asset, Debug, Clone, TypePath)]
pub struct TomlAsset {
    pub document: toml_edit::Document,
}

impl From<toml_edit::Document> for TomlAsset {
    fn from(document: toml_edit::Document) -> Self {
        TomlAsset { document }
    }
}

impl Deref for TomlAsset {
    type Target = toml_edit::Document;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}

fn into_deserializer(
    doc: &toml_edit::Document,
    path: Option<&str>,
) -> Option<toml_edit::de::Deserializer> {
    match path {
        None => Some(doc.clone().into_deserializer()),
        Some(path) => {
            let src_item = walk_toml(doc.as_item(), path)?;
            let mut doc_for_deserialize = toml_edit::Document::new();
            *doc_for_deserialize.as_item_mut() = src_item.clone();
            Some(doc_for_deserialize.into_deserializer())
        }
    }
}

fn walk_toml<'a>(from: &'a toml_edit::Item, path: &str) -> Option<&'a toml_edit::Item> {
    let mut item = from;
    // TODO: implement something more sophisiticated, preferably from a spec,
    // preferably implemented in toml_edit, maybe based on the syntax of dotted keys.
    for segment in path.split('.') {
        item = item.get(segment)?;
    }
    Some(item)
}

pub struct TomlAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TomlLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [conversion Error](std::str::Utf8Error)
    #[error("Could not interpret as UTF-8: {0}")]
    FormatError(#[from] std::str::Utf8Error),
    /// A [TOML Error](toml_edit::de::Error)
    #[error("Could not parse TOML: {0}")]
    TomlError(#[from] toml_edit::TomlError),
}

impl AssetLoader for TomlAssetLoader {
    type Asset = TomlAsset;
    type Settings = ();
    type Error = TomlLoaderError;

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let string = from_utf8(&bytes)?;
            let doc: toml_edit::Document = string.parse()?;
            Ok(doc.into())
        })
    }
}

pub(crate) fn watch_toml_settings<'de, S: Resource + Deserialize<'de>>(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<LoadedUntypedAsset>>,
    untyped_assets: Res<Assets<LoadedUntypedAsset>>,
    toml_assets: Res<Assets<TomlAsset>>,
    load_settings: Res<SettingsPluginSettings<S>>,
    setting: Option<ResMut<S>>,
) {
    let mut new_setting: Option<S> = default();
    for event in events.read() {
        let id = *match event {
            AssetEvent::Added { id } => id,
            AssetEvent::Modified { id } => id,
            AssetEvent::LoadedWithDependencies { id } => id,
            _ => continue,
        };
        if id != load_settings.doc.id() {
            continue;
        };
        let Some(LoadedUntypedAsset { handle }) = untyped_assets.get(id) else {
            continue;
        };
        if handle.type_id() != TypeId::of::<TomlAsset>() {
            continue;
        }
        let asset = toml_assets
            .get(handle.clone().typed::<TomlAsset>())
            .unwrap();
        let de = into_deserializer(&asset.document, load_settings.label.as_deref());
        new_setting = de.and_then(|de| S::deserialize(de).ok());
    }

    if let Some(s) = new_setting {
        match setting {
            Some(mut v) => {
                *v = s;
            }
            None => {
                commands.insert_resource(s);
            }
        }
    }
}
