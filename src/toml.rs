use std::ops::Deref;
use std::str::from_utf8;

use bevy::asset::{AssetLoader, AsyncReadExt};
use bevy::prelude::*;
use serde::de::IntoDeserializer;

use thiserror::Error;

use crate::asset::IntoPathDeserializer;

pub struct TomlAssetPlugin;

impl Plugin for TomlAssetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(TomlAssetLoader)
            .init_asset::<TomlAsset>();
    }
}

#[derive(Asset, Debug, Default, Clone, TypePath)]
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

impl<'de> IntoDeserializer<'de, toml_edit::de::Error> for TomlAsset {
    type Deserializer = toml_edit::de::Deserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        self.document.into_deserializer()
    }
}

impl<'de> IntoPathDeserializer<'de, toml_edit::de::Error> for TomlAsset {
    fn into_deserializer_at(self, path: &str) -> Option<Self::Deserializer> {
        let item = walk_toml(self.as_item(), path)?;
        let mut doc_for_deserialize = toml_edit::Document::new();
        *doc_for_deserialize.as_item_mut() = item.clone();
        Some(doc_for_deserialize.into_deserializer())
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
