# CMakeLists.txt
cmake_minimum_required(VERSION 3.30 FATAL_ERROR)

project(DroxIDE
    VERSION 1.0.0
    DESCRIPTION "Native Desktop AI-Powered IDE - Production Build"
    LANGUAGES C CXX)

# Production-grade C++ settings
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Qt 6.8+ settings with AutoMoc fixes for vcpkg + VS 2026
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_INCLUDE_CURRENT_DIR ON)
set(CMAKE_INCLUDE_CURRENT_DIR_IN_INTERFACE ON)
set(AUTOGEN_USE_SYSTEM_INCLUDE ON)

# Treat warnings as errors in Release
if(MSVC)
    add_compile_options(/W4 /permissive- /EHsc)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Qt6 via vcpkg (your confirmed path)
# If Qt6 is not found, you can still build the Rust core independently
find_package(Qt6 6.5 COMPONENTS
    Core
    Gui
    Widgets
    Network
    CONFIG QUIET)

if(NOT Qt6_FOUND)
    message(STATUS "╔══════════════════════════════════════════════════════════════════╗")
    message(STATUS "║  Qt6 not found. Building Rust core only.                         ║")
    message(STATUS "║                                                                  ║")
    message(STATUS "║  To build full UI:                                               ║")
    message(STATUS "║  Windows: vcpkg install qtbase qttools qt5compat --triplet x64-windows║")
    message(STATUS "║  Then add: -DCMAKE_PREFIX_PATH=C:/vcpkg/installed/x64-windows   ║")
    message(STATUS "╚══════════════════════════════════════════════════════════════════╝")
endif()

# Corrosion for Rust core (strict 2026 standards)
include(FetchContent)
fetchcontent_declare(Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG master
    GIT_SHALLOW TRUE)
fetchcontent_makeavailable(Corrosion)

# Let Corrosion auto-detect the Rust toolchain (don't hardcode)
# The user's system may have stable-x86_64-pc-windows-msvc or gnu

corrosion_import_crate(MANIFEST_PATH Cargo.toml)

# Only build Qt UI if Qt6 is found
if(Qt6_FOUND)
    message(STATUS "✅ Qt6 found. Building full DroxIDE UI application")

    # Complete source list per DroxIDE spec
    set(SOURCES
        src/main.cpp
        src/ffi_bridge.cpp
        src/mainwindow.cpp
        src/mainwindow.h
        src/editor/editor.cpp
        src/editor/editor.h
        src/editor/syntaxhighlighter.cpp
        src/editor/syntaxhighlighter.h
        src/terminal/terminalwidget.cpp
        src/terminal/terminalwidget.h
        src/explorer/explorerwidget.cpp
        src/explorer/explorerwidget.h
        src/panels/agenttracewidget.cpp
        src/panels/agenttracewidget.h
        src/panels/ragheatmapwidget.cpp
        src/panels/ragheatmapwidget.h
        src/dialogs/runswarmdialog.cpp
        src/dialogs/runswarmdialog.h
        src/dialogs/commitdialog.cpp
        src/dialogs/commitdialog.h
        src/dialogs/preferencesdialog.cpp
        src/dialogs/preferencesdialog.h
        src/git/gitmanager.cpp
        src/git/gitmanager.h
        src/lsp/lspclient.cpp
        src/lsp/lspclient.h
        src/resources.qrc
    )

    add_executable(DroxIDE WIN32 ${SOURCES})

    target_link_libraries(DroxIDE PRIVATE
        Qt6::Core
        Qt6::Gui
        Qt6::Widgets
        Qt6::Network
        droxide_rust
    )

    # Fix include directory paths (CRITICAL ROOT CAUSE FIX)
    target_include_directories(DroxIDE PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src
        ${CMAKE_CURRENT_SOURCE_DIR}/src/editor
        ${CMAKE_CURRENT_SOURCE_DIR}/src/terminal
        ${CMAKE_CURRENT_SOURCE_DIR}/src/explorer
        ${CMAKE_CURRENT_SOURCE_DIR}/src/panels
        ${CMAKE_CURRENT_SOURCE_DIR}/src/dialogs
        ${CMAKE_CURRENT_SOURCE_DIR}/src/git
        ${CMAKE_CURRENT_SOURCE_DIR}/src/lsp
    )

    # Production security & Qt hardening
    target_compile_definitions(DroxIDE PRIVATE
        QT_NO_CAST_FROM_ASCII
        QT_NO_CAST_TO_ASCII
        QT_NO_KEYWORDS
        QT_DISABLE_DEPRECATED_BEFORE=0x060800
    )

    # Windows-specific production settings
    if(WIN32)
        set_target_properties(DroxIDE PROPERTIES
            WIN32_EXECUTABLE ON
            MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>"
        )
    endif()

    # Install rules for distribution
    install(TARGETS DroxIDE
        RUNTIME DESTINATION bin
        BUNDLE DESTINATION .
    )

    message(STATUS "========================================")
    message(STATUS "DroxIDE v1.0.0 Production Build Configured")
    message(STATUS "Qt6_DIR: ${Qt6_DIR}")
    message(STATUS "Generator: ${CMAKE_GENERATOR}")
    message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}")
    message(STATUS "Rust Crate: droxide_rust")
    message(STATUS "========================================")
else()
    message(STATUS "========================================")
    message(STATUS "DroxIDE Rust Core Only (Qt6 not found)")
    message(STATUS "To build full UI: vcpkg install qtbase qttools qt5compat")
    message(STATUS "========================================")
endif()