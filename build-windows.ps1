# DroxIDE - Windows Build Script
# Automatically handles Qt6 installation via vcpkg

param(
    [string]$BuildType = "Release",
    [switch]$SkipQt,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "DroxIDE Windows Build Script" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Check for vcpkg
$vcpkgPath = $env:VCPKG_ROOT
if (-not $vcpkgPath) {
    # Try common locations
    $possiblePaths = @(
        "C:\vcpkg",
        "$env:USERPROFILE\vcpkg",
        "C:\Program Files\vcpkg"
    )
    foreach ($path in $possiblePaths) {
        if (Test-Path "$path\vcpkg.exe") {
            $vcpkgPath = $path
            break
        }
    }
}

if (-not $vcpkgPath -or -not (Test-Path "$vcpkgPath\vcpkg.exe")) {
    Write-Host "[1/4] Installing vcpkg..." -ForegroundColor Yellow
    
    if (Test-Path "C:\vcpkg") {
        $vcpkgPath = "C:\vcpkg"
    } else {
        Write-Host "Cloning vcpkg to C:\vcpkg..." -ForegroundColor Gray
        git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
        $vcpkgPath = "C:\vcpkg"
    }
    
    Write-Host "Bootstrapping vcpkg..." -ForegroundColor Gray
    Push-Location $vcpkgPath
    & .\bootstrap-vcpkg.bat
    Pop-Location
    
    $env:VCPKG_ROOT = $vcpkgPath
    [Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkgPath, "User")
} else {
    Write-Host "[1/4] vcpkg found at: $vcpkgPath" -ForegroundColor Green
}

$vcpkgExe = "$vcpkgPath\vcpkg.exe"

if (-not $SkipQt) {
    Write-Host "[2/4] Installing Qt6 via vcpkg..." -ForegroundColor Yellow
    Write-Host "This may take 10-30 minutes depending on your internet connection." -ForegroundColor Gray
    
    & $vcpkgExe install qtbase qttools qt5compat --triplet x64-windows
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install Qt6. You can still build the Rust core with --SkipQt" -ForegroundColor Red
        exit 1
    }
    
    $qt6Dir = "$vcpkgPath\installed\x64-windows\share\Qt6"
    Write-Host "Qt6 installed at: $qt6Dir" -ForegroundColor Green
} else {
    Write-Host "[2/4] Skipping Qt6 (building Rust core only)" -ForegroundColor Yellow
}

# Check for CMake
Write-Host "[3/4] Checking CMake..." -ForegroundColor Yellow
$cmakeVersion = cmake --version 2>$null
if (-not $cmakeVersion) {
    Write-Host "CMake not found. Installing via winget..." -ForegroundColor Gray
    winget install Kitware.CMake
} else {
    Write-Host "CMake found: $cmakeVersion" -ForegroundColor Green
}

# Clean build directory if requested
if ($Clean -and (Test-Path "build")) {
    Write-Host "Cleaning build directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force build
}

# Configure and build
Write-Host "[4/4] Configuring and building DroxIDE..." -ForegroundColor Yellow

# Initialize MSVC 64-bit environment
$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($vsPath) {
    $vsvarsPath = "$vsPath\VC\Auxiliary\Build\vcvars64.bat"
    if (Test-Path $vsvarsPath) {
        Write-Host "Setting up MSVC 64-bit environment..." -ForegroundColor Gray
        cmd /c "`"$vsvarsPath`" && set" | ForEach-Object {
            if ($_ -match "^(.*?)=(.*)$") {
                [Environment]::SetEnvironmentVariable($matches[1], $matches[2])
            }
        }
    }
}

$cmakeArgs = @(
    "-B", "build",
    "-DCMAKE_BUILD_TYPE=$BuildType",
    "-DRust_COMPILER=C:\Users\droxa\.cargo\bin\rustc.exe",
    "-DRust_CARGO=C:\Users\droxa\.cargo\bin\cargo.exe"
)

if (Test-Path "$vcpkgPath\scripts\buildsystems\vcpkg.cmake") {
    $cmakeArgs += "-DCMAKE_TOOLCHAIN_FILE=$vcpkgPath\scripts\buildsystems\vcpkg.cmake"
}

if ($qt6Dir) {
    $cmakeArgs += "-DCMAKE_PREFIX_PATH=$qt6Dir"
}

Write-Host "Running: cmake $($cmakeArgs -join ' ')" -ForegroundColor Gray
cmake @cmakeArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "CMake configuration failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Building..." -ForegroundColor Gray
cmake --build build --config $BuildType -j $env:NUMBER_OF_PROCESSORS

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host "Build successful!" -ForegroundColor Green
    Write-Host "=====================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Executable: build\$BuildType\DroxIDE.exe" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To run:" -ForegroundColor Gray
    Write-Host "  .\build\$BuildType\DroxIDE.exe" -ForegroundColor White
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
