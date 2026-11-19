//! Worksheet-level structures and formatting properties.
#[derive(Debug, Clone, Default)]
pub enum FCellValue {
    /// Empty cell.
    #[default]
    Empty,
    /// String cell.
    String(String),
    /// Numeric cell.
    Number(f64),
    /// Boolean cell.
    Bool(bool),
}
/// Horizontal and vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FCellAlignment {
    #[default]
    General,
    Left,
    Center,
    Right,
    Justify,
    Top,
    Bottom,
}
/// Border side style.
#[derive(Debug, Clone, Default)]
pub struct FCellBorder {
    /// Border style (thin, medium, thick, dashed, etc.).
    pub style: Option<String>,
    /// Border color.
    pub color: Option<String>,
}
/// Cell borders.
#[derive(Debug, Clone, Default)]
pub struct FCellBorders {
    pub top: Option<FCellBorder>,
    pub left: Option<FCellBorder>,
    pub bottom: Option<FCellBorder>,
    pub right: Option<FCellBorder>,
}
/// Cell fill.
#[derive(Debug, Clone, Default)]
pub struct FCellFill {
    /// Fill pattern type (solid, gray125, etc.).
    pub pattern: Option<String>,
    /// Foreground color.
    pub fg_color: Option<String>,
    /// Background color.
    pub bg_color: Option<String>,
}
/// Cell font.
#[derive(Debug, Clone, Default)]
pub struct FCellFont {
    /// Font name.
    pub name: Option<String>,
    /// Font size in points.
    pub size: Option<f64>,
    /// Bold flag.
    pub bold: Option<bool>,
    /// Italic flag.
    pub italic: Option<bool>,
    /// Underline flag.
    pub underline: Option<bool>,
    /// Font color.
    pub color: Option<String>,
}
/// Number format code.
#[derive(Debug, Clone, Default)]
pub struct FCellFormat {
    /// Number format id.
    pub num_fmt_id: Option<u32>,
    /// Format code, e.g. `"yyyy-mm-dd"`.
    pub format_code: Option<String>,
}
/// Cell style definition.
#[derive(Debug, Clone, Default)]
pub struct FCellStyle {
    /// Style name.
    pub name: Option<String>,
    /// Number format.
    pub format: Option<FCellFormat>,
    /// Font.
    pub font: Option<FCellFont>,
    /// Fill.
    pub fill: Option<FCellFill>,
    /// Borders.
    pub borders: Option<FCellBorders>,
    /// Alignment.
    pub alignment: Option<FCellAlignment>,
    /// Raw `<xf>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
