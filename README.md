# Bevy settings loader

A bevy plugin offering loading and hot-reloading of settings resources from
configuration files.

## Usage

In an app:

```rust
use serde::Deserialize;
use bevy::prelude::*;
use bevy_settings::SettingsPlugin;

#[derive(Resource, Deserialize)]
struct ControlSettings {
    deadzone: f32,
    acceleration: f32,
    max_speed: f32,
}

fn main() {
    App::new()
        .add_plugins(SettingsPlugin::<ControlSettings>::load("controls.toml"))
        .add_systems(Update, handle_controls);
}

fn handle_controls(
    // Use `Option` to handle the system being run before the settings have been
    // loaded and inserted into the world.
    settings: Option<Res<ControlSettings>>,
    mut players: Query<(&mut Player)>,
    axes: Res<Axis<GamepadAxis>>,
) {
    //...
}
```

Or in a plugin:

```rust
use serde::Deserialize;
use bevy::prelude::*;
use bevy_settings::SettingsPlugin;

pub struct EnemySpawnerPlugin;

#[derive(Default, Resource, Deserialize)]
struct EnemySpawnerSettings {
    entity_count: u32,
}

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        // Use init_resource to ensure the default settings are present even if
        // the configuration file is unavailable.
        app.init_resource::<EnemySpawnerSettings>()
            .add_plugins(SettingsPlugin::<EnemySpawnerSettings>::load("enemy_spawner.toml"));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    settings: Res<EnemySpawnerSettings>,
) {
    //...
}
```
