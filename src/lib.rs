#[cfg(feature = "toml")]
pub mod toml;

#[cfg(feature = "json")]
pub mod json;

mod asset;
mod plugin;

pub use asset::IntoPathDeserializer;
pub use plugin::SettingsPlugin;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(1, 4);
    }
}
