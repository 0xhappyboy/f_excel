//! Table.
#[derive(Debug, Clone, Default)]
pub struct FTable {
    /// Table name.
    pub name: String,
    /// Table display name.
    pub display_name: Option<String>,
    /// Table range.
    pub range: String,
    /// Table columns.
    pub columns: Vec<FTableColumn>,
    /// Table style name.
    pub style_name: Option<String>,
    /// Raw `<table>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Table column definition.
#[derive(Debug, Clone, Default)]
pub struct FTableColumn {
    /// Column name.
    pub name: String,
    /// Column id.
    pub id: Option<u32>,
    /// Raw `<tableColumn>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
