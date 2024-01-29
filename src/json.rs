use std::any::TypeId;
use std::ops::Deref;
use std::str::from_utf8;

use bevy::asset::{AssetLoader, AsyncReadExt, LoadedUntypedAsset};
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

use crate::plugin::SettingsPluginSettings;

pub struct JsonAssetPlugin;

impl Plugin for JsonAssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(JsonAssetLoader)
            .init_asset::<JsonAsset>();
    }
}

#[derive(Asset, Debug, Clone, TypePath)]
pub struct JsonAsset {
    document: serde_json::Value,
}

impl From<serde_json::Value> for JsonAsset {
    fn from(doc: serde_json::Value) -> Self {
        JsonAsset { document: doc }
    }
}

impl Deref for JsonAsset {
    type Target = serde_json::Value;
    fn deref(&self) -> &Self::Target {
        &self.document
    }
}

pub struct JsonAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonLoaderError {
    /// An [IO Error](std::io::Error)
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    /// A [conversion Error](std::str::Utf8Error)
    #[error("Could not interpret as UTF-8: {0}")]
    FormatError(#[from] std::str::Utf8Error),
    /// A [TOML Error](toml_edit::de::Error)
    #[error("Could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl AssetLoader for JsonAssetLoader {
    type Asset = JsonAsset;
    type Settings = ();
    type Error = JsonLoaderError;

    fn extensions(&self) -> &[&str] {
        &["json"]
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
            let doc: serde_json::Value = serde_json::from_str(string)?;
            Ok(doc.into())
        })
    }
}

pub(crate) fn watch_json_settings<'de, S: Resource + Deserialize<'de>>(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<LoadedUntypedAsset>>,
    untyped_assets: Res<Assets<LoadedUntypedAsset>>,
    json_assets: Res<Assets<JsonAsset>>,
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
        if handle.type_id() != TypeId::of::<JsonAsset>() {
            continue;
        }
        let asset = json_assets
            .get(handle.clone().typed::<JsonAsset>())
            .unwrap();
        let mut doc = asset.deref();
        if let Some(path) = &load_settings.label {
            match doc.pointer(path) {
                Some(val) => doc = val,
                None => continue,
            };
        };
        new_setting = S::deserialize(doc.clone()).ok();
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
