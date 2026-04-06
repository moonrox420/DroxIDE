// src/ffi_bridge.cpp
// Bridge between Qt C++ and Rust core
// This file provides C-compatible FFI functions that mainwindow.cpp calls

#include <string>
#include <vector>
#include <cstring>

// External Rust FFI functions (linked from Rust staticlib)
extern "C" {
    int init_orchestrator();
    char* run_swarm(const char* prompt, const char** context_files, int file_count);
    int accept_diff(const char* diff_id);
    int reject_diff(const char* diff_id, const char* feedback);
    char* get_orchestrator_state();
    char* get_metrics_summary();
}

// Helper to convert Rust string to C string (caller must free)
static char* rust_string_to_c(const char* rust_str) {
    if (!rust_str) return nullptr;
    size_t len = strlen(rust_str);
    char* c_str = (char*)malloc(len + 1);
    if (c_str) {
        memcpy(c_str, rust_str, len);
        c_str[len] = '\0';
    }
    return c_str;
}

extern "C" {

void init_orchestrator_ffi() {
    try {
        init_orchestrator();
    } catch (const std::exception& e) {
        fprintf(stderr, "Failed to initialize orchestrator: %s\n", e.what());
    }
}

char* run_swarm_ffi(const char* prompt, const char** context_files, int file_count) {
    try {
        return run_swarm(prompt, context_files, file_count);
    } catch (const std::exception& e) {
        fprintf(stderr, "Swarm execution failed: %s\n", e.what());
        return nullptr;
    }
}

void accept_diff_ffi(const char* diff_id) {
    try {
        accept_diff(diff_id);
    } catch (const std::exception& e) {
        fprintf(stderr, "Failed to accept diff: %s\n", e.what());
    }
}

void reject_diff_ffi(const char* diff_id, const char* feedback) {
    try {
        reject_diff(diff_id, feedback);
    } catch (const std::exception& e) {
        fprintf(stderr, "Failed to reject diff: %s\n", e.what());
    }
}

char* get_metrics_summary_ffi() {
    try {
        return get_metrics_summary();
    } catch (const std::exception& e) {
        fprintf(stderr, "Failed to get metrics: %s\n", e.what());
        return nullptr;
    }
}

void free_string_ffi(char* str) {
    if (str) {
        free(str);
    }
}

} // extern "C"
