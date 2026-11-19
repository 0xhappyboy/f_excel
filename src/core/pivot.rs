//! Pivot table.
#[derive(Debug, Clone, Default)]
pub struct FPivotTable {
    /// Pivot table name.
    pub name: String,
    /// Pivot table range.
    pub range: String,
    /// Pivot fields.
    pub fields: Vec<FPivotField>,
    /// Raw `<pivotTableDefinition>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Pivot field definition.
#[derive(Debug, Clone, Default)]
pub struct FPivotField {
    /// Field name.
    pub name: String,
    /// Field axis.
    pub axis: Option<String>,
    /// Raw `<pivotField>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
