#[derive(Debug, Clone, Default)]
pub struct FWorkbookProtection {
    /// Workbook password hash.
    pub workbook_password: Option<String>,
    /// Lock structure flag.
    pub lock_structure: Option<bool>,
    /// Lock windows flag.
    pub lock_windows: Option<bool>,
    /// Lock revision flag.
    pub lock_revision: Option<bool>,
    /// Raw `<workbookProtection>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Sheet protection.
#[derive(Debug, Clone, Default)]
pub struct FSheetProtection {
    /// Sheet password hash.
    pub password: Option<String>,
    /// Sheet flag.
    pub sheet: Option<bool>,
    /// Objects flag.
    pub objects: Option<bool>,
    /// Scenarios flag.
    pub scenarios: Option<bool>,
    /// Format cells flag.
    pub format_cells: Option<bool>,
    /// Format columns flag.
    pub format_columns: Option<bool>,
    /// Format rows flag.
    pub format_rows: Option<bool>,
    /// Insert columns flag.
    pub insert_columns: Option<bool>,
    /// Insert rows flag.
    pub insert_rows: Option<bool>,
    /// Insert hyperlinks flag.
    pub insert_hyperlinks: Option<bool>,
    /// Delete columns flag.
    pub delete_columns: Option<bool>,
    /// Delete rows flag.
    pub delete_rows: Option<bool>,
    /// Select locked cells flag.
    pub select_locked_cells: Option<bool>,
    /// Sort flag.
    pub sort: Option<bool>,
    /// Auto filter flag.
    pub auto_filter: Option<bool>,
    /// Pivot tables flag.
    pub pivot_tables: Option<bool>,
    /// Select unlocked cells flag.
    pub select_unlocked_cells: Option<bool>,
    /// Raw `<sheetProtection>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
