#[cfg(feature = "toml")]
pub mod toml;

#[cfg(feature = "json")]
pub mod json;

mod plugin;

pub use plugin::SettingsPlugin;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(1, 4);
    }
}
