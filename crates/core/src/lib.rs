#![forbid(unsafe_code)]

pub const APPLICATION_NAME: &str = "Memelith";

/// Application state owned by the Memelith frontend.
///
/// waifu-sensor is an independent library and CLI; applications opt into it by
/// constructing the library with their own SQLite connection and asset paths.
#[derive(Debug, Default)]
pub struct Memelith;

impl Memelith {
    pub fn new() -> Self {
        Self
    }

    pub fn status_summary(&self) -> &'static str {
        "Ready"
    }
}

#[cfg(test)]
mod tests {
    use super::Memelith;

    #[test]
    fn memelith_core_does_not_construct_waifu_sensor_implicitly() {
        assert_eq!(Memelith::new().status_summary(), "Ready");
    }
}
