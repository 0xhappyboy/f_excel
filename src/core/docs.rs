//! Root workbook.
use super::sheet::{FCellStyle, FCellValue};
use super::workbook::{FDefinedName, FWorkbookProperties, FWorkbookView};
use crate::FPackage;
/// Worksheet cell content.
#[derive(Debug, Clone, Default)]
pub struct FCell {
    /// Raw cell reference, e.g. `"A1"`.
    pub reference: String,
    /// Cell value.
    pub value: FCellValue,
    /// Style index into the workbook style table.
    pub style_id: Option<u32>,
    /// Formula text, if present.
    pub formula: Option<String>,
    /// Raw `<c>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Row of cells in a worksheet.
#[derive(Debug, Clone, Default)]
pub struct FRow {
    /// Row number, 1-based.
    pub index: u32,
    /// Cells in this row.
    pub cells: Vec<FCell>,
    /// Row height in points.
    pub height: Option<f64>,
    /// Row style index.
    pub style_id: Option<u32>,
    /// Raw `<row>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Worksheet data.
#[derive(Debug, Clone, Default)]
pub struct FSheet {
    /// Sheet name.
    pub name: String,
    /// Sheet relationship id.
    pub relationship_id: String,
    /// Archive path of the sheet part.
    pub path: String,
    /// Rows in this sheet.
    pub rows: Vec<FRow>,
    /// Column widths keyed by column index (1-based).
    pub column_widths: Vec<(u32, f64)>,
    /// Merged cell ranges, e.g. `"A1:B2"`.
    pub merged_cells: Vec<String>,
    /// Sheet-level styles.
    pub styles: Vec<FCellStyle>,
    /// Freeze panes.
    pub freeze: Option<super::freeze::FFreezePane>,
    /// Auto filter.
    pub autofilter: Option<super::autofilter::FAutoFilter>,
    /// Data validations.
    pub data_validations: Vec<super::datavalidation::FDataValidation>,
    /// Conditional formatting rules.
    pub conditional_formats: Vec<super::conditional::FConditionalFormat>,
    /// Print settings.
    pub print: Option<super::print::FPrintSettings>,
    /// Sheet protection.
    pub protection: Option<super::protection::FSheetProtection>,
    /// Sheet hyperlinks.
    pub hyperlinks: Vec<super::hyperlink::FHyperlink>,
    /// Sheet comments.
    pub comments: Vec<super::comment::FComment>,
    /// Sheet drawings.
    pub drawings: Vec<super::drawing::FDrawing>,
    /// Sheet tables.
    pub tables: Vec<super::table::FTable>,
    /// Pivot tables.
    pub pivots: Vec<super::pivot::FPivotTable>,
    /// Raw `<worksheet>` XML preserved for elements not modeled above.
    pub raw_xml: Option<String>,
}
/// Root of the Excel workbook data model.
#[derive(Debug, Clone, Default)]
pub struct FWorkbook {
    /// Package-level parts that are not modeled explicitly.
    pub package: FPackage,
    /// Workbook-level properties.
    pub properties: FWorkbookProperties,
    /// Workbook view settings.
    pub view: FWorkbookView,
    /// Defined names.
    pub defined_names: Vec<FDefinedName>,
    /// Worksheets in the workbook.
    pub sheets: Vec<FSheet>,
    /// Shared strings table.
    pub shared_strings: Vec<String>,
    /// Workbook-level styles.
    pub styles: Vec<FCellStyle>,
    /// Workbook theme.
    pub theme: Option<String>,
    /// Workbook protection.
    pub protection: Option<super::protection::FWorkbookProtection>,
    /// Whether the workbook has been modified since it was loaded.
    pub dirty: bool,
}
impl FWorkbook {
    /// Creates an empty in-memory Excel workbook.
    pub fn new_empty() -> Self {
        let mut wb = FWorkbook::default();
        wb.package.main_part = "xl/workbook.xml".to_string();
        wb.properties = FWorkbookProperties::default();
        wb.view = FWorkbookView::default();
        wb
    }
    /// Loads an Excel workbook from a file on disk.
    pub fn with_path<P: AsRef<std::path::Path>>(path: P) -> crate::FExcelResult<Self> {
        let data = std::fs::read(path)?;
        Self::with_bytes(&data)
    }
    /// Parses an Excel workbook from its raw `.xlsx` bytes.
    pub fn with_bytes(data: &[u8]) -> crate::FExcelResult<Self> {
        crate::io::reader::parse_workbook(data)
    }
    /// Serializes the workbook back into `.xlsx` bytes.
    pub fn to_bytes(&self) -> crate::FExcelResult<Vec<u8>> {
        crate::io::writer::serialize_workbook(self)
    }
    /// Marks the workbook as modified.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    /// Clears the modified flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
    /// Returns the plain text content of all sheets.
    pub fn plain_text(&self) -> String {
        let mut text = String::new();
        for sheet in &self.sheets {
            text.push_str(&format!("Sheet: {}\n", sheet.name));
            for row in &sheet.rows {
                for cell in &row.cells {
                    match &cell.value {
                        FCellValue::String(v) => text.push_str(v),
                        FCellValue::Number(v) => text.push_str(&v.to_string()),
                        FCellValue::Bool(v) => text.push_str(if *v { "TRUE" } else { "FALSE" }),
                        FCellValue::Empty => {}
                    }
                    text.push('\t');
                }
                text.push('\n');
            }
        }
        text
    }
    /// Returns the number of sheets in the workbook.
    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }
    /// Returns the number of rows across all sheets.
    pub fn row_count(&self) -> usize {
        self.sheets.iter().map(|s| s.rows.len()).sum()
    }
    /// Returns the number of cells across all sheets.
    pub fn cell_count(&self) -> usize {
        self.sheets
            .iter()
            .flat_map(|s| s.rows.iter())
            .map(|r| r.cells.len())
            .sum()
    }
}
