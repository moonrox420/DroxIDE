// src/ffi_bridge.cpp
// C++ FFI bridge to Rust staticlib
#include <string>
#include <vector>
#include <cstdio>

extern "C" {
    int rust_init_orchestrator();
    char* rust_run_swarm(const char* prompt, const char** context_files, int file_count);
    int rust_accept_diff(const char* diff_id);
    int rust_reject_diff(const char* diff_id, const char* feedback);
    char* rust_get_metrics_summary();
    void rust_free_string(char* s);
}

namespace DroxBridge {

int init_orchestrator() {
    return rust_init_orchestrator();
}

char* run_swarm(const char* prompt, const std::vector<const char*>& files) {
    if (!prompt) return nullptr;
    return rust_run_swarm(
        prompt,
        const_cast<const char**>(files.data()),
        static_cast<int>(files.size())
    );
}

void accept_diff(const std::string& diff_id) {
    if (diff_id.empty()) return;
    rust_accept_diff(diff_id.c_str());
}

void reject_diff(const std::string& diff_id, const std::string& feedback) {
    if (diff_id.empty()) return;
    rust_reject_diff(diff_id.c_str(), feedback.c_str());
}

char* get_metrics() {
    return rust_get_metrics_summary();
}

void free_rust_string(char* str) {
    if (str) {
        rust_free_string(str);
    }
}

} // namespace DroxBridge
