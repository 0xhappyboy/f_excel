//! Style.
use super::sheet::{FCellAlignment, FCellBorder, FCellBorders, FCellFill, FCellFont, FCellFormat};
/// A complete style definition.
#[derive(Debug, Clone, Default)]
pub struct FStyle {
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
    /// Protection flags.
    pub protection: Option<FStyleProtection>,
    /// Raw `<xf>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Style protection flags.
#[derive(Debug, Clone, Default)]
pub struct FStyleProtection {
    /// Locked flag.
    pub locked: Option<bool>,
    /// Hidden flag.
    pub hidden: Option<bool>,
}
