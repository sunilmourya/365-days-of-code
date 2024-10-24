use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

pub(crate) fn extract_zip_file(zip_path: &Path) -> Result<(), String> {
    let zip_file = File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

    // Get the output directory based on the ZIP file's location
    let output_directory = zip_path
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;

    // Unzip files
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|e| e.to_string())?;
        let output_file_path = output_directory.join(file.name());

        if let Some(parent) = output_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let mut output_file =
            File::create(&output_file_path).map_err(|e| e.to_string())?;
        io::copy(&mut file, &mut output_file).map_err(|e| e.to_string())?;
    }

    // Remove the ZIP file after extraction
    fs::remove_file(zip_path).map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn create_zip_from_folder(
    folder_path: &Path,
) -> Result<PathBuf, String> {
    // Check if the folder exists
    if !folder_path.is_dir() {
        return Err(format!("The path {:?} is not a directory.", folder_path));
    }

    // Create the ZIP file path
    let zip_file_path = folder_path.with_extension("zip");

    // Create a new ZIP file
    let zip_file =
        fs::File::create(&zip_file_path).map_err(|e| e.to_string())?;
    let mut zip_writer = ZipWriter::new(zip_file);

    for entry in WalkDir::new(folder_path) {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();

        // Get the relative path from the base folder
        let relative_path =
            entry_path.strip_prefix(folder_path).map_err(|e| e.to_string())?;
        let relative_path_str = relative_path.to_string_lossy();

        if entry_path.is_dir() {
            // Use () as the FileOptionExtension for directories
            zip_writer
                .add_directory::<_, ()>(
                    relative_path_str.as_ref(),
                    FileOptions::default(),
                )
                .map_err(|e| e.to_string())?;
        } else {
            // Add a file to the ZIP, specify the type explicitly
            let options: FileOptions<'_, ()> = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .large_file(false);

            zip_writer
                .start_file(relative_path_str.as_ref(), options)
                .map_err(|e| e.to_string())?;

            let mut file =
                fs::File::open(entry_path).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut zip_writer).map_err(|e| e.to_string())?;
        }
    }

    zip_writer.finish().map_err(|e| e.to_string())?;
    Ok(zip_file_path)
}
