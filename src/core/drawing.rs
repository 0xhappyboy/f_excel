//! Drawing related.
#[derive(Debug, Clone, Default)]
pub struct FDrawing {
    /// Drawing name.
    pub name: String,
    /// Drawing type.
    pub drawing_type: Option<String>,
    /// Drawing anchor range.
    pub anchor: Option<String>,
    /// Raw `<drawing>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
