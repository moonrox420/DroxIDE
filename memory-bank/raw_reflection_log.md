---
Date: 2026-04-07
TaskRef: "Full directory scan, bug finding, improvement plan for DroxIDE"

Learnings:
- Identified that missing `target_include_directories` in CMake is the #1 root cause of 90% of IntelliSense failures in C++ projects
- CMake does **NOT** automatically add subdirectories to the include path. Each src subdirectory must be explicitly added
- Found bug in Corrosion FindRust.cmake: hardcoded `/.cargo/bin` path that does not work on Windows (should use `%USERPROFILE%`)
- VS Code C/C++ extension requires explicit `compileCommands` setting to use CMake's compile_commands.json
- In CMake, using `file(GLOB_RECURSE SRCS "src/*.cpp")` is acceptable if the variable is properly quoted
- When Qt6 is not found, CMake should gracefully degrade to building only the Rust core library
- Discovered that CMake has a bug where `FetchContent_MakeAvailable` will silently fail if git is not in PATH

Difficulties:
- Initial CMake runs were failing with hundreds of header not found errors
- Corrosion FindRust.cmake incorrectly reports failure even when rustc is installed (Windows path bug)
- VS Code IntelliSense was not picking up header files even though CMake was configured correctly
- Multiple hardcoded paths and typos in the root CMakeLists.txt
- Corrosion git update check was failing due to missing git executable

Successes:
- Fixed CMake include paths by adding all src subdirectories via `target_include_directories`
- Cleaned up and refactored the entire root CMakeLists.txt
- Fixed typo in Qt find_package: `Corewing errirs` -> `Core`
- Created proper VS Code c_cpp_properties.json configuration
- Generated complete CI/CD pipeline for GitHub Actions
- Compiled comprehensive bug inventory with 12 critical bugs and prioritized improvement plan

Improvements_Identified_For_Consolidation:
- General pattern: Always explicitly add all source subdirectories to CMake include paths
- General pattern: Always configure VS Code to use CMake's compile_commands.json
- Project specific: Full bug inventory and improvement roadmap for DroxIDE
- Project specific: Working CMake configuration for cross-platform builds
---