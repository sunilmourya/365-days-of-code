# Check if Chocolatey is installed
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Chocolatey..."
    # Temporarily set execution policy to Bypass for this session
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072;

    # Download and execute the Chocolatey installation script
    try {
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    } catch {
        Write-Host "Failed to download or execute Chocolatey installation script. Exiting..."
        exit 1

    }

    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Host "Failed to install Chocolatey. Exiting..."
        exit 1
    }
}

# Install LLVM/Clang using Chocolatey
Write-Host "Installing the latest version of LLVM/Clang..."
choco install llvm -y

# Check if LLVM/Clang was successfully installed
if ($?) {
    Write-Host "LLVM/Clang installed successfully."
} else {
    Write-Host "Failed to install LLVM/Clang. Exiting..."
    exit 1
}

# Set the LLVM/Clang binary path to the system PATH environment variable
$llvmPath = "C:\Program Files\LLVM\bin"

if (-not (Test-Path $llvmPath)) {
    Write-Host "LLVM binary path not found. Please check the installation path."
    exit 1
}

# Update the PATH environment variable
$env:Path += ";$llvmPath"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::Machine)

# Verify if the environment variable is set
if ($env:Path -like "*$llvmPath*") {
    Write-Host "LLVM/Clang environment variable has been set successfully."
} else {
    Write-Host "Failed to set the environment variable."
    exit 1
}

# Verify LLVM/Clang installation
Write-Host "Verifying LLVM/Clang installation..."
try {
    Start-Process -NoNewWindow -Wait -FilePath "clang" -ArgumentList "--version"
} catch {
    Write-Host "Failed to run clang to verify installation. Exiting..."
    exit 1
}

Write-Host "Installation completed."
