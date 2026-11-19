//! Workbook-level.
#[derive(Debug, Clone, Default)]
pub struct FWorkbookProperties {
    /// Workbook code name.
    pub code_name: Option<String>,
    /// Date system (1900 or 1904).
    pub date1904: Option<bool>,
    /// Raw `<workbookPr>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Workbook view settings.
#[derive(Debug, Clone, Default)]
pub struct FWorkbookView {
    /// Active tab index.
    pub active_tab: Option<u32>,
    /// Show horizontal scroll flag.
    pub show_horizontal_scroll: Option<bool>,
    /// Show vertical scroll flag.
    pub show_vertical_scroll: Option<bool>,
    /// Show sheet tabs flag.
    pub show_sheet_tabs: Option<bool>,
    /// Raw `<workbookView>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Defined name entry.
#[derive(Debug, Clone, Default)]
pub struct FDefinedName {
    /// Defined name.
    pub name: String,
    /// Formula text.
    pub text: String,
    /// Local sheet id, if scoped to a sheet.
    pub local_sheet_id: Option<u32>,
    /// Raw `<definedName>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
