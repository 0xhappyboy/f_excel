//! Error types for the Excel workbook engine.
#[derive(Debug)]
pub enum FExcelError {
    /// I/O failure when reading or writing files.
    Io(std::io::Error),
    /// The archive could not be opened or a required entry is missing.
    Zip(zip::result::ZipError),
    /// The XML content is malformed or does not match the expected schema.
    Xml(String),
    /// The requested operation is not valid for the current workbook state.
    Invalid(String),
}
impl std::fmt::Display for FExcelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FExcelError::Io(e) => write!(f, "io error: {}", e),
            FExcelError::Zip(e) => write!(f, "zip error: {}", e),
            FExcelError::Xml(e) => write!(f, "xml error: {}", e),
            FExcelError::Invalid(e) => write!(f, "invalid operation: {}", e),
        }
    }
}
impl std::error::Error for FExcelError {}
impl From<std::io::Error> for FExcelError {
    fn from(e: std::io::Error) -> Self {
        FExcelError::Io(e)
    }
}
impl From<zip::result::ZipError> for FExcelError {
    fn from(e: zip::result::ZipError) -> Self {
        FExcelError::Zip(e)
    }
}
pub type FExcelResult<T> = Result<T, FExcelError>;
