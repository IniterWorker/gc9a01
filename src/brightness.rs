//! Display brightness

/// Struct that holds display brightness
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Brightness {
    pub(crate) brightness: u8,
}

impl Default for Brightness {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl Brightness {
    ///Dimmest predefined brightness level
    pub const DIMMEST: Self = Self::custom(0x00);

    /// Dim predefined brightness level
    pub const DIM: Self = Self::custom(0x2F);

    /// Medium predefined brightness level
    pub const NORMAL: Self = Self::custom(0x5F);

    /// Bright predefined brightness level
    pub const BRIGHT: Self = Self::custom(0x9F);

    /// Brightest predefined brightness level
    pub const BRIGHTEST: Self = Self::custom(0xFF);

    /// Create a new `Brightness` from a custom raw input
    ///
    /// # Notes
    ///
    /// It should be checked what is the relationship between this written value and output brightness of the display.
    /// This relationship is defined on the display module specification.
    /// In principle, the relationship is that `00h` value means the lowest brightness and `FFh` value means the highest brightness.
    ///
    #[must_use]
    pub const fn custom(brightness: u8) -> Self {
        Self { brightness }
    }

    /// Returns the brightness as a `u8`.
    ///
    /// # Note
    /// This function is provided for convenience only. It does not read the brightness from the hardware driver.
    #[must_use]
    pub const fn brightness(&self) -> u8 {
        self.brightness
    }
}
