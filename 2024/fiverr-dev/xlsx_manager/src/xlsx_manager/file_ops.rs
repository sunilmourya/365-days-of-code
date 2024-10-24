use log::info;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Custom error type for file operations
#[derive(Debug, Error)]
pub enum FileOpsError {
    #[error("Invalid source file path: {0}")]
    InvalidFilePath(String),
    #[error("IO error")]
    IoError(#[from] io::Error),
}

/// Ensures that the specified directory exists. If it doesn't exist, create it.
pub fn create_directory_if_missing(
    directory_path: &str,
) -> Result<(), FileOpsError> {
    let directory = Path::new(directory_path);
    if !directory.exists() {
        fs::create_dir_all(directory).map_err(FileOpsError::IoError)?;
        info!("Created directory: {}", directory_path);
    } else {
        info!("Directory already exists: {}", directory_path);
    }
    Ok(())
}

/// Determines whether the provided file path has an Excel-compatible extension (.xls or .xlsx).
pub fn has_excel_extension(file_path: &Path) -> bool {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("xls") | Some("xlsx") => true,
        _ => {
            info!("Unsupported file extension for: {:?}", file_path);
            false
        }
    }
}

/// Generates the full output file path in the specified `output_directory` for the processed file.
/// The output file will have the same extension (.xls or .xlsx) as the source file.
pub fn generate_output_file_path(
    source_file_path: &Path,
    output_directory: &str,
) -> Result<PathBuf, FileOpsError> {
    // Ensure the output directory exists
    create_directory_if_missing(output_directory)?;

    // Extract the base file name (without extension) from the source file path
    let file_name_without_extension = source_file_path
        .file_stem()
        .ok_or_else(|| {
            FileOpsError::InvalidFilePath(format!(
                "No file name found in path: {:?}",
                source_file_path
            ))
        })?
        .to_str()
        .ok_or_else(|| {
            FileOpsError::InvalidFilePath(format!(
                "Invalid UTF-8 sequence in filename: {:?}",
                source_file_path
            ))
        })?;

    // Get the original file extension, ensuring its valid
    // let file_extension = source_file_path
    //     .extension()
    //     .and_then(|ext| ext.to_str())
    //     .ok_or_else(|| {
    //         FileOpsError::InvalidFilePath(format!(
    //             "No file extension found in path: {:?}",
    //             source_file_path
    //         ))
    //     })?;

    // Construct the new file name with the same extension
    // let new_file_name =
    //     format!("{}.{}", file_name_without_extension, file_extension);
    let new_file_name = format!("{}.xlsx", file_name_without_extension);

    // Generate the full output path by combining the output directory and new file name
    let output_file_path = Path::new(output_directory).join(new_file_name);

    info!("Generated output file path: {:?}", output_file_path);
    Ok(output_file_path)
}
