use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ProcessResponse {
    pub job_id: String,
    pub time_taken: String,
    pub num_rows_deleted: usize,
    pub zip_file_name: String,
}
