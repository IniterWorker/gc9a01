//! Display Rotation

/// Screen Rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayRotation {
    #[default]
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}
