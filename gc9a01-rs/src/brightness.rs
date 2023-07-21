//! Display brightness

/// Struct that holds display brightness
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Brightness {
    pub(crate) brightness: u8,
}

impl Default for Brightness {
    fn default() -> Self {
        Brightness::NORMAL
    }
}

impl Brightness {
    ///Dimmest predefined brightness level
    pub const DIMMEST: Brightness = Brightness::custom(0x00);

    /// Dim predefined brightness level
    pub const DIM: Brightness = Brightness::custom(0x2F);

    /// Medium predefined brightness level
    pub const NORMAL: Brightness = Brightness::custom(0x5F);

    /// Bright predefined brightness level
    pub const BRIGHT: Brightness = Brightness::custom(0x9F);

    /// Brightest predefined brightness level
    pub const BRIGHTEST: Brightness = Brightness::custom(0xFF);

    const fn custom(brightness: u8) -> Self {
        Self { brightness }
    }
}
