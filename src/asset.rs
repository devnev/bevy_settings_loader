use std::any::TypeId;

use bevy::{asset::LoadedUntypedAsset, prelude::*};
use serde::{
    de::{value, Error, IntoDeserializer},
    Deserialize,
};

use crate::plugin::SettingsPluginSettings;

pub trait IntoPathDeserializer<'de, E: Error = value::Error>: IntoDeserializer<'de, E> {
    fn into_deserializer_at(self, path: &str) -> Option<Self::Deserializer>;
}

pub(crate) fn watch_settings_asset<
    'de,
    S: Resource + Deserialize<'de>,
    A: Asset + IntoPathDeserializer<'de, E> + Clone,
    E: Error,
>(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<LoadedUntypedAsset>>,
    untyped_assets: Res<Assets<LoadedUntypedAsset>>,
    watched_assets: Res<Assets<A>>,
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
        if handle.type_id() != TypeId::of::<A>() {
            continue;
        }
        let asset = watched_assets.get(handle.clone().typed::<A>()).unwrap();
        let de = match &load_settings.label {
            Some(path) => asset.clone().into_deserializer_at(path),
            None => Some(asset.clone().into_deserializer()),
        };
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
