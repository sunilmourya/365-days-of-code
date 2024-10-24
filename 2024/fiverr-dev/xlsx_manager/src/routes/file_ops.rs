use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn create_guid_directory(
    guid: &str,
) -> Result<PathBuf, std::io::Error> {
    let directory_path = PathBuf::from(format!("upload/{}", guid));
    // Create the directory, propagating any potential errors
    fs::create_dir_all(&directory_path)?;
    Ok(directory_path)
}

pub(crate) fn list_excel_files_in_directory(
    directory_path: &Path,
) -> Result<Vec<String>, String> {
    let mut excel_file_names = Vec::new();

    // Read the directory and collect all .xlsx and .xls file names
    let entries = fs::read_dir(directory_path).map_err(|e| e.to_string())?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        // Check if the file extension is either .xlsx or .xls
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if extension == "xlsx" || extension == "xls" {
                if let Some(file_name) =
                    path.file_name().and_then(|name| name.to_str())
                {
                    excel_file_names.push(file_name.to_string());
                }
            }
        }
    }

    if excel_file_names.is_empty() {
        return Err(
            "No Excel files (.xlsx or .xls) found in the specified directory."
                .to_string(),
        );
    }

    Ok(excel_file_names)
}
