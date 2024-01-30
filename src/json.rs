
use std::ops::Deref;
use std::str::from_utf8;

use bevy::asset::{AssetLoader, AsyncReadExt};
use bevy::prelude::*;
use serde::de::IntoDeserializer;

use thiserror::Error;

use crate::asset::IntoPathDeserializer;


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

impl<'de> IntoDeserializer<'de, serde_json::Error> for JsonAsset {
    type Deserializer = serde_json::Value;

    fn into_deserializer(self) -> Self::Deserializer {
        self.document.into_deserializer()
    }
}

impl<'de> IntoPathDeserializer<'de, serde_json::Error> for JsonAsset {
    fn into_deserializer_at(self, path: &str) -> Option<Self::Deserializer> {
        self.pointer(path).cloned()
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
