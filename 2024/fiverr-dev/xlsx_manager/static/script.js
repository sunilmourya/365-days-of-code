document.addEventListener("DOMContentLoaded", () => {
    const fileList = [];
    const fileListContainer = document.getElementById("fileList");
    const totalFilesLabel = document.getElementById("totalFiles");
    let job_id = undefined;

    // Input for Excel files
    const excelInput = document.createElement("input");
    excelInput.type = "file";
    excelInput.accept = ".xlsx,.xls";
    excelInput.style.display = "none";

    // Input for Zip files
    const zipInput = document.createElement("input");
    zipInput.type = "file";
    zipInput.accept = ".zip";
    zipInput.style.display = "none";

    // Input for Folder containing only .xlsx and .xls files
    const folderInput = document.createElement("input");
    folderInput.type = "file";
    folderInput.webkitdirectory = true; // Allows folder selection
    folderInput.style.display = "none";

    document.body.appendChild(excelInput);
    document.body.appendChild(zipInput);
    document.body.appendChild(folderInput);

    // Handle "Add Excel" button click
    document.getElementById("addExcel").addEventListener("click", () => {
        excelInput.click();
    });

    // Handle "Add Zip" button click
    document.getElementById("addZip").addEventListener("click", () => {
        zipInput.click();
    });

    // Handle "Add Folder" button click
    document.getElementById("addFolder").addEventListener("click", () => {
        folderInput.click();
    });

    // Excel input change event (single files)
    excelInput.addEventListener("change", () => {
        const files = Array.from(excelInput.files);
        addValidFiles(files, [".xlsx", ".xls"]);
    });

    // Zip input change event (only .zip files)
    zipInput.addEventListener("change", () => {
        const files = Array.from(zipInput.files);
        addValidFiles(files, [".zip"]);
    });

    // Folder input change event (only .xlsx and .xls files)
    folderInput.addEventListener("change", () => {
        const files = Array.from(folderInput.files);
        const validFiles = files.filter(file => file.name.endsWith(".xlsx") || file.name.endsWith(".xls"));
        addValidFiles(validFiles, [".xlsx", ".xls"]);
    });

    // Function to add valid files
    function addValidFiles(files, validExtensions) {
        files.forEach((file) => {
            const fileExtension = file.name.substring(file.name.lastIndexOf('.')).toLowerCase();
            if (validExtensions.includes(fileExtension) && !fileList.some(f => f.name === file.name)) {
                fileList.push(file);
            }
        });
        updateFileList();
    }

    // Function to update the file list displayed
    function updateFileList() {
        fileListContainer.innerHTML = "";
        fileList.forEach((file, index) => {
            const listItem = document.createElement("li");
            listItem.textContent = file.name;

            const buttonsDiv = document.createElement("div");
            buttonsDiv.className = "file-buttons";

            const removeButton = document.createElement("button");
            removeButton.textContent = "X";
            removeButton.className = "clear";
            removeButton.addEventListener("click", () => removeFile(index));

            buttonsDiv.appendChild(removeButton);
            listItem.appendChild(buttonsDiv);
            fileListContainer.appendChild(listItem);
        });

        totalFilesLabel.textContent = `Total ${fileList.length} files`;
    }

    function removeFile(index) {
        fileList.splice(index, 1);
        updateFileList();
    }

    // Use "submit" event on the form
    document.getElementById("fileForm").addEventListener("submit", async (event) => {
        event.preventDefault(); // Prevent default form submission
        updateStatus("Form submitted successfully!", true);
        const deleteRows = document.getElementById("deleteRows").value;
        if (!deleteRows || fileList.length === 0) {
            updateStatus("Please select files and specify the number of rows to delete.", true, "red");
            return;
        }

        const formData = new FormData();
        fileList.forEach((file) => {
            formData.append("files", file, file.name);
        });

        job_id = "";

        try {
            // First API call: Upload the files
            const uploadStartTime = Date.now(); // Record start time for upload
            const uploadResponse = await fetch("http://localhost:8080/upload", {
                method: "POST",
                body: formData,
            });

            const jobId = await uploadResponse.text();
            job_id = jobId;

            console.log("(POST) Upload Response: ", job_id);
            const uploadEndTime = Date.now(); // Record end time for upload
            const uploadDuration = uploadEndTime - uploadStartTime; // Calculate duration
            updateStatus(`Files uploaded successfully! Job ID: ${jobId} (Took ${uploadDuration} ms)`, true);

            // Second API call: Process the job
            const processStartTime = Date.now(); // Record start time for processing
            const processResponse = await fetch("http://localhost:8080/process", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    job_id: job_id,
                    num_rows_to_delete: parseInt(deleteRows),
                }),
            });

            console.log("Process response:", processResponse);

            const processData = await processResponse.json();

            console.log("(POST)Process Response: ", processData);

            const processEndTime = Date.now(); // Record end time for processing
            const processDuration = processEndTime - processStartTime; // Calculate duration
            updateStatus(`Processing completed successfully!\n Job ID: ${jobId} (Took ${processDuration} ms)`, true);

            // Third API call: Download
            const downloadResponse = await fetch("http://localhost:8080/download", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    file_url: processData.zip_file_name,
                }),
            })

            console.log("(POST)Download Response: ", downloadResponse);

            if (downloadResponse.ok) {
                console.log("Download success:", downloadResponse.statusText);
                const blob = await downloadResponse.blob();
                const downloadLink = URL.createObjectURL(blob);
                const downloadElement = document.getElementById('downloadLink');
                downloadElement.href = downloadLink;

                const zip_file = processData.zip_file_name.split(/[/\\]/).pop(); // Get the last part of full path
                downloadElement.download = zip_file;
                downloadElement.innerText = zip_file;
                document.getElementById('downloadLinkContainer').style.display = 'block';
            } else {
                console.error("Download failed:", downloadResponse.statusText);
            }

        } catch (error) {
            console.error("Error:", error);
            updateStatus("Failed to process files. Please try again.", true, "red");
        }
    });

    document.getElementById("clearForm").addEventListener("click", async () => {
        fileList.length = 0;
        updateFileList();
        document.getElementById("deleteRows").value = "";
        document.getElementById('downloadLinkContainer').style.display = 'none';

        updateStatus("Form cleared.", true);

        // Hide the status message after a few seconds
        setTimeout(() => {
            updateStatus("", false);
        }, 1000);

        // On clear button press - Clear data(delete upload/job-id dir) and delete data from server.
        try {
            const removeResponse = await fetch(`http://localhost:8080/remove/${job_id}`, {
                method: "DELETE",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            console.log("(DELETE) Remove Response: ", removeResponse);

            if (removeResponse.ok) {
                const result = await removeResponse.text(); // or await removeResponse.json() depending on the response format
                console.log("Job removed successfully:", result);
            }
        } catch (error) {
            console.error("Error removing job:", error);
        }
    });

    // Function to update the status message
    function updateStatus(message, show = true, color = "black") {
        const statusContainer = document.getElementById("statusContainer");
        const statusMessage = document.getElementById("statusMessage");

        // Update the message text
        statusMessage.textContent = message;
        statusMessage.style.color = color;

        // Show or hide the status container based on the 'show' parameter
        if (show) {
            statusContainer.style.display = "block"; // Show the status container
        } else {
            statusContainer.style.display = "none"; // Hide the status container
        }
    }
});
