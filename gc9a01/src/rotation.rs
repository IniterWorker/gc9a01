//! Display Rotation

/// Screen Rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayRotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Default for DisplayRotation {
    fn default() -> Self {
        Self::Rotate0
    }
}
