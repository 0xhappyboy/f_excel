//! Excel workbook parsing engine.
mod core;
mod error;
mod io;
mod pkg;
mod xml;
pub use core::docs::{FCell, FRow, FSheet, FWorkbook};
pub use core::sheet::{
    FCellAlignment, FCellBorder, FCellBorders, FCellFill, FCellFont, FCellFormat, FCellStyle,
    FCellValue,
};
pub use core::workbook::{FDefinedName, FWorkbookProperties, FWorkbookView};
pub use error::{FExcelError, FExcelResult};
pub use io::*;
pub use pkg::{FPackage, FPackagePart};
pub use xml::*;
