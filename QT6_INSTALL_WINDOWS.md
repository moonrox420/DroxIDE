# Qt6 Installation Guide for Windows

## Quick Fix for Your Error

Your CMake error means Qt6 is not installed. Here are 3 solutions:

---

## Solution 1: Automated Script (Easiest)

Run the included build script:

```powershell
.\build-windows.ps1
```

This will automatically:
1. Install vcpkg if missing
2. Install Qt6 via vcpkg (~15 min download)
3. Configure CMake with correct paths
4. Build DroxIDE

---

## Solution 2: Manual vcpkg Installation

```powershell
# 1. Install vcpkg
git clone https://github.com/microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# 2. Install Qt6 (takes 10-30 minutes)
.\vcpkg.exe install qt6-base qt6-tools qt6-network --triplet x64-windows

# 3. Build DroxIDE
cd C:\Users\droxa\DroxIDE
cmake -B build -DCMAKE_TOOLCHAIN_FILE=C:\vcpkg\scripts\buildsystems\vcpkg.cmake
cmake --build build --config Release
```

---

## Solution 3: Offline Installer (Fastest if you have bandwidth)

1. Download Qt Online Installer: https://www.qt.io/download-qt-installer
2. Run installer, select Qt 6.8.x for MSVC 2022 64-bit
3. Note the installation path (e.g., `C:\Qt\6.8.0\msvc2022_64`)
4. Build with:

```powershell
cmake -B build -DCMAKE_PREFIX_PATH="C:\Qt\6.8.0\msvc2022_64"
cmake --build build --config Release
```

---

## Temporary Workaround: Build Rust Core Only

If you want to test the Rust code immediately without Qt6:

```powershell
# Build Rust core
cargo build --release

# Run tests
cargo test
```

This won't build the UI, but validates all the Rust logic.

---

## Verify Installation

After installing Qt6, verify:

```powershell
# Check vcpkg
C:\vcpkg\vcpkg.exe list

# Should show Qt6 packages
cmake --find-package -DNAME=Qt6 -DCOMPILER_ID=MSVC -DLANGUAGE=CXX -DMODE=EXIST

# Build DroxIDE
cmake -B build -DCMAKE_TOOLCHAIN_FILE=C:\vcpkg\scripts\buildsystems\vcpkg.cmake
cmake --build build --config Release
```

---

## Common Issues

### "Cannot find Qt6Config.cmake"
**Fix:** Use vcpkg toolchain file:
```powershell
cmake -B build -DCMAKE_TOOLCHAIN_FILE=C:\vcpkg\scripts\buildsystems\vcpkg.cmake
```

### "vcpkg install takes too long"
**Fix:** Use pre-built Qt binaries from qt.io instead

### "MSVC compiler errors"
**Fix:** Install Visual Studio 2022 with "Desktop development with C++" workload

---

## Recommended Setup

For development on Windows:
- Visual Studio 2022 (Community edition is free)
- vcpkg for Qt6
- Rust via rustup-init.exe
- CMake via installer or winget

This gives you the full development environment.
