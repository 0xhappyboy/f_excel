#[cfg(test)]
mod tests {
    use f_excel::{FCell, FCellValue, FRow, FSheet, FWorkbook};
    fn save(wb: &FWorkbook, file_name: &str) -> std::path::PathBuf {
        #[cfg(target_os = "windows")]
        let dir = std::path::PathBuf::from(r"C:\Users\admin\Downloads");
        #[cfg(not(target_os = "windows"))]
        let dir = std::env::temp_dir();
        let path = dir.join(file_name);
        let bytes = wb.to_bytes().expect("failed to serialize workbook");
        std::fs::write(&path, &bytes).expect("failed to write workbook");
        let meta = std::fs::metadata(&path).expect("output file missing");
        assert!(meta.len() > 0, "output file is empty: {file_name}");
        path
    }
    #[test]
    fn create_workbook() {
        let mut wb = FWorkbook::new_empty();
        let mut sheet = FSheet {
            name: "Sheet1".into(),
            relationship_id: "rId2".into(),
            path: "xl/worksheets/sheet1.xml".into(),
            ..Default::default()
        };
        let mut row = FRow {
            index: 1,
            ..Default::default()
        };
        row.cells.push(FCell {
            reference: "A1".into(),
            value: FCellValue::String("Hello".into()),
            ..Default::default()
        });
        row.cells.push(FCell {
            reference: "B1".into(),
            value: FCellValue::Number(123.0),
            ..Default::default()
        });
        row.cells.push(FCell {
            reference: "C1".into(),
            value: FCellValue::Bool(true),
            ..Default::default()
        });
        sheet.rows.push(row);
        wb.sheets.push(sheet);
        let path = save(&wb, "workbook.xlsx");
        println!("workbook written to {}", path.display());
    }
}
