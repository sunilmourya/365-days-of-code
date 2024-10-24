# Features
**File Upload:** Supports .xlsx, .xls (Excel), .zip files, and folders containing Excel files.

**File Validation:** Ensures only the correct file types are added, avoiding duplicates.

**File Display:** Lists the selected files and updates the count dynamically.

**Processing Workflow:**
Upload files to the server.
Specify the number of rows to delete from each file's first sheet.
Trigger processing and wait for results.
Download the processed files as a Zip file.

**Form Reset:** Clears selected files, resets form data, and deletes server-side job data.
Status Updates: Provides real-time feedback for each stage (uploading, processing, downloading).

# UI Components
**Add Excel:** Button to select .xlsx or .xls files.

**Add Zip:** Button to select .zip files.

**Add Folder:** Button to select folders containing Excel files.

**File List:** Displays the list of selected files and the total number.

**Delete Rows Input:** Input field to specify the number of rows to delete from the first sheet.

**Submit Button:** Uploads files, initiates the server-side process, and allows downloading processed files.

**Clear Button:** Clears the form and deletes the job on the server.

**Status Message**: Displays current status (e.g., files uploaded, job processed).

**Download Link:** Allows downloading processed files as a Zip file.


# API Endpoints
### 1. Upload Files
```
   Endpoint: /upload
   Method: POST
   Description: Uploads Excel and Zip files to the server for processing.
```

#### Request
**Content-Type:** multipart/form-data

**Body:** Upload multiple Excel and/or Zip files (files[]).

#### Response
Returns a unique job ID for the uploaded files.

##### Example
```
curl -X POST http://localhost:8080/upload \
-F "files=@file1.xlsx" \
-F "files=@file2.zip"
```

### 2. Process Files
```
   Endpoint: /process
   Method: POST
   Description: Deletes the specified number of rows from the first sheet of each Excel file and compresses the processed files into a Zip file.
```
#### Request
**Content-Type:** application/json
**Body:**
job_id: Unique job ID returned from _**/upload**_
num_rows_to_delete: Number of rows to delete from each Excel file's first sheet.

#### Response
Returns the path to the generated Zip file.

##### Example
```
curl -X POST http://localhost:8080/process \
-H "Content-Type: application/json" \
-d '{"job_id": "abc123", "num_rows_to_delete": 5}'
```

### 3. Download Processed Files
```
   Endpoint: /download
   Method: POST
   Description: Downloads the processed files as a Zip file.
```
#### Request
**Content-Type:** application/json
**Body:**
file_url: Path to the processed Zip file returned from /process.

#### Response
Returns the Zip file for download.

##### Example
```
curl -X POST http://localhost:8080/download \
-H "Content-Type: application/json" \
-d '{"file_url": "/path/to/processed/file.zip"}' \
--output processed_files.zip
```

### 4. Remove Job
```
   Endpoint: /remove/{job_id}
   Method: DELETE
   Description: Deletes the uploaded files and job data associated with the specified job ID.
```
#### Request
Path Parameter: job_id - Unique job ID returned from **_/upload_**.

#### Response
Returns a success message indicating that the job has been deleted.

##### Example
```
curl -X DELETE http://localhost:8080/remove/abc123 \
-H "Content-Type: application/json"
```

# Run application
1. cargo run
2. open in browser 127.0.0.1:8080
3. _index.html_ can be opened _"/"_ hitting endpoint

## Suggested improvements
- Better error handling
- More modular code(JS/Rust)
- UI improvements
- Test cases
