use crate::xlsx_manager::file_ops::{
    create_directory_if_missing, generate_output_file_path, has_excel_extension,
};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use log::{error, info};
use rayon::prelude::*;
use std::path::Path;
use std::{error, thread};
use xlsxwriter::{Format, Worksheet};

// Creates and returns a new Excel workbook for writing output.
// The workbook is created at the specified `file_path`.
// The function returns the new workbook or an error if the creation fails.
pub fn create_new_workbook(
    file_path: &Path,
) -> Result<xlsxwriter::Workbook, Box<dyn error::Error>> {
    Ok(xlsxwriter::Workbook::new(file_path.to_str().unwrap())?)
}

// Processes all Excel files in the given `source_folder` and saves the processed
// files to the `target_folder`. Files are processed in parallel using the `rayon` crate.
pub fn process_excel_files_parallel(
    source_folder: &str, // Path to the source folder containing Excel files
    target_folder: &str, // Path to the target folder where processed files will be saved
    files: &[String],    // Specific files to process
    delete_first_n_rows: usize, // Delete first N rows
) -> Result<(), Box<dyn error::Error>> {
    // Ensure the target folder exists or create it
    create_directory_if_missing(target_folder)
        .expect("Error with target folder.");

    info!("Folders\nSource: {}\nTarget: {}", source_folder, target_folder);

    // Process files in parallel using Rayon
    files.par_iter().for_each(|file| {
        let source_file_path = Path::new(source_folder).join(file);

        // Validate if the file is an Excel file
        if !has_excel_extension(&source_file_path) {
            error!("Skipping invalid excel file: {}", file);
            return;
        }

        // Process the file and handle any errors
        if let Err(err) = process_single_excel(
            &source_file_path,
            target_folder,
            delete_first_n_rows,
        ) {
            error!(
                "Error processing file {}: {}",
                source_file_path.display(),
                err
            );
        }
    });

    Ok(())
}

// Processes a single Excel file located at the given `path`, and writes the processed
// data to a new file in the `target_folder`. It opens the Excel file, reads the data,
// processes it, and saves it in the target folder with the same name.
pub fn process_single_excel(
    source_file_path: &Path, // Path to the source Excel file
    target_folder: &str, // Path to the target folder where the processed file will be saved
    delete_first_n_rows: usize, // Delete first N rows
) -> Result<(), Box<dyn error::Error>> {
    info!(
        "Processing file: {}, {:?}",
        source_file_path.display(),
        thread::current().id()
    );
    // Open the Excel workbook automatically detecting its format
    let mut workbook = open_workbook_auto(source_file_path)?;
    // Get the first sheet
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err("No sheets found in the excel file.".into());
    }

    let first_sheet = &sheet_names[0];
    // Create a format for date cells
    let mut date_format = Format::new();
    date_format.set_num_format("yyyy-mm-dd hh:mm:ss");

    if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
        // Generate the path for the processed file
        let target_file_path =
            generate_output_file_path(source_file_path, target_folder)?;

        // Create a new Excel workbook for output
        let workbook_out = create_new_workbook(&target_file_path)?;

        // Add a worksheet to the new workbook
        let mut sheet = workbook_out.add_worksheet(Some(first_sheet))?;

        process_rows(&range, &mut sheet, &date_format, delete_first_n_rows)?;
        workbook_out.close()?;
        info!("File processed and saved: {}", target_file_path.display());
    } else {
        return Err("Failed to read the first sheet.".into());
    }
    Ok(())
}

// Processes rows and cells from the provided `range` (Excel data) and writes them
// to the `sheet` in the new workbook. The first 7 rows are skipped (header rows),
// and each cell is processed individually based on its type.
pub fn process_rows(
    range: &calamine::Range<Data>, // Data range from the source Excel file
    sheet: &mut Worksheet,         // Worksheet to write the processed rows to
    date_format: &Format,          // Format for date cells
    del_first_n_rows: usize,
) -> Result<(), Box<dyn error::Error>> {
    // Skip the first N rows (assumed to be headers) and process the rest
    for (row_idx, row) in range.rows().skip(del_first_n_rows).enumerate() {
        // Process each cell in the row
        for (col_idx, cell) in row.iter().enumerate() {
            process_cell(
                cell,
                row_idx as u32,
                col_idx as u16,
                sheet,
                date_format,
            )?;
        }
    }
    Ok(())
}

/// Processes a single cell from the source Excel data and writes it to the corresponding
/// position in the output worksheet. The cell type is checked and handled accordingly
/// (e.g., numbers, dates, strings). For date-like numbers, conversion is applied.
fn process_cell(
    cell: &Data,           // The data type of the cell to be processed
    row_idx: u32,          // Row index in the output worksheet
    col_idx: u16,          // Column index in the output worksheet
    sheet: &mut Worksheet, // The worksheet to write the cell to
    date_format: &Format,  // Date format for formatting date cells
) -> Result<(), Box<dyn error::Error>> {
    match cell {
        Data::Float(f) => {
            sheet.write_number(row_idx, col_idx, *f, None)?;
        }

        Data::String(s) => {
            sheet.write_string(row_idx, col_idx, s, None)?;
        }
        Data::Int(i) => {
            sheet.write_number(row_idx, col_idx, *i as f64, None)?;
        }
        Data::Bool(b) => {
            sheet.write_boolean(row_idx, col_idx, *b, None)?;
        }
        Data::DateTime(excel_dt) => {
            let serial_dt: f64 = excel_dt.as_f64();

            if let Some(naive_dt) = excel_serial_to_naive_datetime(serial_dt) {
                let xlsx_dt = xlsxwriter::worksheet::DateTime {
                    year: naive_dt.year() as i16,
                    month: naive_dt.month() as i8,
                    day: naive_dt.day() as i8,
                    hour: naive_dt.hour() as i8,
                    min: naive_dt.minute() as i8,
                    second: naive_dt.second() as f64,
                };

                sheet.write_datetime(
                    row_idx,
                    col_idx,
                    &xlsx_dt,
                    Some(&date_format),
                )?;
            } else {
                error!("Error converting datetime to NaiveDateTime: excel_dt => {}, serial_dt => {}", excel_dt, serial_dt);
                // Handle the case where the conversion fails (optional)
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid Excel serial date",
                )));
            }
        }
        Data::Error(_) | Data::Empty => {
            sheet.write_blank(row_idx, col_idx, None)?; // Ignore error or empty cells
        }
        _ => {} // For other data types (e.g., empty cells), nothing is done
    }
    Ok(())
}

/// Converts an Excel date (stored as a floating-point number) to a `chrono::NaiveDateTime`.
/// The Excel date starts from December 30, 1899, which is used as the base date for conversion.
pub fn excel_serial_to_naive_datetime(
    excel_date: f64,
) -> Option<NaiveDateTime> {
    // Excels base date is 1899-12-30
    let base_date = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    // Separate the date into whole days and seconds (fractional part)
    let days = excel_date.trunc() as i64;
    let seconds_in_day = (excel_date.fract() * 86400.0).round() as i64;

    // Add the days and seconds to the base date
    base_date
        .and_hms_opt(0, 0, 0)?
        .checked_add_signed(Duration::days(days))?
        .checked_add_signed(Duration::seconds(seconds_in_day))
}
