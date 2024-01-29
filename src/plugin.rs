use bevy::asset::{AssetPath, LoadedUntypedAsset};
use bevy::prelude::*;
use serde::Deserialize;
use std::marker::PhantomData;

use crate::json::JsonAssetPlugin;
use crate::toml::TomlAssetPlugin;

#[derive(Debug)]
pub struct SettingsPlugin<S> {
    path: String,
    _marker: PhantomData<S>,
}

impl<S> SettingsPlugin<S> {
    pub fn load(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            _marker: default(),
        }
    }
}

#[derive(Resource, Debug, Reflect)]
#[reflect(from_reflect = false)]
pub(crate) struct SettingsPluginSettings<S> {
    pub path: String,
    pub label: Option<String>,
    pub doc: Handle<LoadedUntypedAsset>,
    #[reflect(ignore)]
    _marker: PhantomData<S>,
}

impl<'de, S: Resource + Deserialize<'de>> Plugin for SettingsPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        let parsed = AssetPath::parse(&self.path);
        app.insert_resource(SettingsPluginSettings::<S> {
            path: parsed.without_label().to_string(),
            label: parsed.label().map(|l| l.to_string()),
            doc: default(),
            _marker: default(),
        })
        .add_systems(Startup, load_settings::<S>);

        #[cfg(feature = "toml")]
        {
            if !app.is_plugin_added::<TomlAssetPlugin>() {
                app.add_plugins(TomlAssetPlugin);
            };
            use crate::toml::watch_toml_settings;
            app.add_systems(Update, watch_toml_settings::<S>);
        }

        #[cfg(feature = "json")]
        {
            if !app.is_plugin_added::<JsonAssetPlugin>() {
                app.add_plugins(JsonAssetPlugin);
            };
            use crate::json::watch_json_settings;
            app.add_systems(Update, watch_json_settings::<S>);
        }
    }
}

fn load_settings<'de, S: Resource + Deserialize<'de>>(
    mut res: ResMut<SettingsPluginSettings<S>>,
    asset_server: Res<AssetServer>,
) {
    res.doc = asset_server.load_untyped(&res.path);
}
