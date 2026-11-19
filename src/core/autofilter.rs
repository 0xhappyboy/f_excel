//! Auto filter.
#[derive(Debug, Clone, Default)]
pub struct FAutoFilter {
    /// Filter range.
    pub range: String,
    /// Filter columns.
    pub columns: Vec<FAutoFilterColumn>,
    /// Raw `<autoFilter>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Auto filter column.
#[derive(Debug, Clone, Default)]
pub struct FAutoFilterColumn {
    /// Column index.
    pub col_id: u32,
    /// Filter values.
    pub filters: Vec<String>,
    /// Raw `<filterColumn>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
