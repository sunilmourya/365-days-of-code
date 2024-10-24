use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct NumberOfRowsToDeleteRequest {
    pub job_id: String,
    pub num_rows_to_delete: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ZipFileDownloadRequest {
    pub file_url: String,
}
