use crate::routes::file_ops::{
    create_guid_directory, list_excel_files_in_directory,
};
use crate::routes::request::{
    NumberOfRowsToDeleteRequest, ZipFileDownloadRequest,
};
use crate::routes::response::ProcessResponse;
use crate::routes::zip_ops::{create_zip_from_folder, extract_zip_file};
use crate::xlsx_manager::xlsx_manager::process_excel_files_parallel;
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use futures::StreamExt;
use log::{debug, error, info};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;

pub(crate) fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/upload").route(web::post().to(upload)))
        .service(web::resource("/process").route(web::post().to(process)))
        .service(
            web::resource("/remove/{job_id}").route(web::delete().to(remove)),
        )
        .service(web::resource("/download").route(web::post().to(download)));
}

async fn upload(
    http_req: HttpRequest,
    mut payload: Multipart,
) -> impl Responder {
    // Log all headers in debug mode
    #[cfg(debug_assertions)]
    for (header_name, header_value) in http_req.headers() {
        info!("{}: {:?}", header_name, header_value);
    }

    // Generate a unique GUID for the session
    let guid = Uuid::new_v4().to_string();
    info!("Generated GUID: {}", guid);

    // Create a folder using the GUID
    let folder_path = match create_guid_directory(&guid) {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to create folder: {}", e);
            return HttpResponse::InternalServerError()
                .body("Failed to create upload folder");
        }
    };

    // Process each field in the multipart payload
    while let Some(field_result) = payload.next().await {
        // Handle payload errors
        let mut field = match field_result {
            Ok(field) => field,
            Err(e) => {
                error!("Payload error: {}", e);
                return HttpResponse::InternalServerError()
                    .body(format!("Payload error: {}", e));
            }
        };

        // Check if content disposition is present
        let content_disposition = match field.content_disposition() {
            Some(disposition) => disposition,
            None => {
                error!("Missing content disposition");
                return HttpResponse::BadRequest()
                    .body("Content disposition is missing");
            }
        };

        // Extract the filename from content disposition
        let filename = match content_disposition.get_filename() {
            Some(name) => name,
            None => {
                error!("Missing filename in content disposition");
                return HttpResponse::BadRequest().body("Filename is missing");
            }
        };

        let filepath = Path::new(&folder_path).join(&filename);
        debug!("Saving file to: {:?}", filepath);

        // Create a file for writing
        let mut file = match fs::File::create(&filepath) {
            Ok(f) => f,
            Err(e) => {
                error!("File creation error: {}", e);
                return HttpResponse::InternalServerError()
                    .body(format!("File create error: {}", e));
            }
        };

        // Write chunks into the file
        while let Some(chunk_result) = field.next().await {
            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!("Chunk read error: {}", e);
                    return HttpResponse::InternalServerError()
                        .body(format!("File chunk error: {}", e));
                }
            };

            if let Err(e) = file.write_all(&chunk) {
                error!("File write error: {}", e);
                return HttpResponse::InternalServerError()
                    .body(format!("File write error: {}", e));
            }
        }
    }

    // Return the GUID as the response body
    HttpResponse::Ok().body(guid)
}

async fn process(
    row_deletion_request: web::Json<NumberOfRowsToDeleteRequest>,
) -> impl Responder {
    let start_time = Instant::now(); // Start timing

    let job_id = &row_deletion_request.job_id;
    let rows_to_delete = row_deletion_request.num_rows_to_delete as usize;
    info!(
        "Received process request for job_id: {}, rows_to_delete: {}",
        job_id, rows_to_delete
    );

    let job_folder = format!("upload/{}", job_id);
    let job_folder_path = Path::new(&job_folder);

    // Ensure the job folder exists
    if !job_folder_path.exists() {
        let error_message = format!(
            "Job folder for Job-Id '{}' not found in the 'upload' folder",
            job_id
        );
        error!("{}", error_message);
        return HttpResponse::NotFound().body(error_message);
    }

    // Generate a timestamped output folder name
    let timestamp = Utc::now().format("%m%d%y%H%M%S").to_string();
    let processed_output_dir_path =
        job_folder_path.join(format!("firstsheet{}", timestamp));

    // Create the output folder
    if let Err(err) = fs::create_dir(&processed_output_dir_path) {
        let error_message = format!(
            "Failed to create output folder {}: {}",
            processed_output_dir_path.display(),
            err
        );
        error!("{}", error_message);
        return HttpResponse::InternalServerError().body(error_message);
    }

    // Retrieve and unzip any ZIP files in the job folder (parallelize ZIP extraction)
    let zip_files: Vec<_> = match fs::read_dir(&job_folder_path) {
        Ok(entries) => entries
            .filter_map(|entry| {
                entry.ok().and_then(|dir_entry| {
                    let path_buf = dir_entry.path();
                    if path_buf.extension().and_then(|s| s.to_str())
                        == Some("zip")
                    {
                        Some(path_buf)
                    } else {
                        None
                    }
                })
            })
            .collect(),
        Err(err) => {
            let error_message = format!(
                "Failed to read the directory '{}': {}",
                job_folder_path.display(),
                err
            );
            error!("{}", error_message);
            return HttpResponse::InternalServerError().body(error_message);
        }
    };

    // Unzip each found ZIP file in parallel (if there are any)
    zip_files.iter().for_each(|zip_file_path| {
        if let Err(err) = extract_zip_file(zip_file_path) {
            error!(
                "Failed to unzip and extract '{}': {}",
                zip_file_path.display(),
                err
            );
        }
    });

    // Process Excel files in the job folder
    let (num_rows_deleted, processed_zip_file_name) =
        match list_excel_files_in_directory(&job_folder_path) {
            Ok(excel_files) => {
                if !excel_files.is_empty() {
                    info!(
                        "Found {} Excel files to process in '{}'",
                        excel_files.len(),
                        processed_output_dir_path.display()
                    );

                    // Process the Excel files in parallel, deleting rows
                    if let Err(err) = process_excel_files_parallel(
                        job_folder_path.to_str().unwrap(), // Source folder (unzipped files)
                        processed_output_dir_path.to_str().unwrap(), // Output folder (for processed files)
                        &excel_files,   // List of Excel files
                        rows_to_delete, // Number of rows to delete
                    ) {
                        let error_message = format!(
                            "Error reading Excel files in '{}': {}",
                            processed_output_dir_path.display(),
                            err
                        );
                        error!("{}", error_message);
                        return HttpResponse::InternalServerError()
                            .body(error_message);
                    }

                    info!(
                        "Successfully processed {} Excel files",
                        excel_files.len()
                    );

                    // Create a zip from the output folder
                    let zip_name = match create_zip_from_folder(
                        &processed_output_dir_path,
                    ) {
                        Ok(path_buf) => {
                            Some(path_buf.to_string_lossy().into_owned())
                        }
                        Err(err) => {
                            error!("Failed to create ZIP file: {}", err);
                            None
                        }
                    };

                    (rows_to_delete, zip_name)
                } else {
                    let error_message = format!(
                        "No Excel files found in folder {}",
                        processed_output_dir_path.display()
                    );
                    info!("{}", error_message);
                    return HttpResponse::NotFound().body(error_message);
                }
            }
            Err(err) => {
                let error_message = format!(
                    "Error reading Excel files in '{}': {}",
                    processed_output_dir_path.display(),
                    err
                );
                error!("{}", error_message);
                return HttpResponse::InternalServerError().body(error_message);
            }
        };

    // Calculate and log the time taken
    let elapsed_time = start_time.elapsed();
    let time_taken = format!("{:.2?}", elapsed_time);
    info!("Processing time: {:.2?}", time_taken);

    // Create response
    let response = ProcessResponse {
        job_id: job_id.to_string(),
        time_taken,
        num_rows_deleted,
        zip_file_name: processed_zip_file_name
            .unwrap_or_else(|| "".to_string()), // Handle None case
    };

    HttpResponse::Ok().json(response)
}

async fn remove(job_id: web::Path<String>) -> impl Responder {
    let job_id = job_id.into_inner();

    info!("Received delete request for job_id: {}", job_id);

    let job_folder = format!("upload/{}", job_id);
    let job_folder_path = Path::new(&job_folder);

    if job_folder_path.exists() {
        // Log the folder existence
        debug!("Folder for job_id {} exists", &job_id);

        // Delete the folder and its contents
        match fs::remove_dir_all(&job_folder_path) {
            Ok(_) => {
                // Folder deleted successfully
                let response_message = format!(
                    "Folder for Job-Id {} deleted successfully",
                    job_id
                );
                info!("Ok: {}", &response_message);
                HttpResponse::Ok().body(response_message)
            }
            Err(err) => {
                // Error deleting folder
                let error_message = format!("Failed to delete folder: {}", err);
                error!("Err: {}", error_message);
                HttpResponse::InternalServerError().body(error_message)
            }
        }
    } else {
        // Folder doesn't exist, return an error
        let error_message =
            format!("Folder for Job-Id {} not found in upload folder", job_id);
        debug!("NotFound: {}", error_message);
        HttpResponse::NotFound().body(error_message)
    }
}

async fn download(
    zip_file_url: web::Json<ZipFileDownloadRequest>,
) -> impl Responder {
    let zip_file_url_path = &zip_file_url.file_url;
    info!("Received download request: {}", zip_file_url_path);

    // Check if the file exists
    if let Ok(metadata) = fs::metadata(&zip_file_url_path) {
        if metadata.is_file() {
            // If the file exists and is valid, serve it
            return match fs::read(&zip_file_url_path) {
                Ok(file_content) => HttpResponse::Ok()
                    .content_type("application/zip")
                    .body(file_content),
                Err(err) => {
                    error!("Failed to read the file: {}", err);
                    HttpResponse::InternalServerError()
                        .body("Failed to read the file")
                }
            };
        }
    }

    // If the file doesn't exist, return a 404
    HttpResponse::NotFound().body("File not found")
}
