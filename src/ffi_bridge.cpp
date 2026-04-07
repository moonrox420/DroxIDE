// src/ffi_bridge.cpp
// src/ffi_bridge.cpp (Refined)
#include <string>
#include <vector>
#include <cstring>
#include <iostream>

// Use the namespace we defined to keep things clean
namespace DroxBridge {

char* run_swarm(const char* prompt, const std::vector<const char*>& files) {
    if (!prompt) return nullptr;
    
    // The cast below resolves the "const char *const *" vs "const char **" mismatch
    // by explicitly passing the data pointer of the vector.
    return rust_run_swarm(
        prompt, 
        const_cast<const char**>(files.data()), 
        static_cast<int>(files.size())
    );
}

} 
/**
 * PRODUCTION NOTE: 
 * We use a dedicated namespace to avoid polluting the global C scope
 * and ensure that the C++ wrappers handle pointer lifecycle strictly.
 */

extern "C" {
    // Low-level symbols linked from the Rust staticlib (.a / .lib)
    // Rust side must use #[no_mangle] and pub extern "C"
    int rust_init_orchestrator();
    char* rust_run_swarm(const char* prompt, const char** context_files, int file_count);
    int rust_accept_diff(const char* diff_id);
    int rust_reject_diff(const char* diff_id, const char* feedback);
    char* rust_get_metrics_summary();
    void rust_free_string(char* s); // Rust must provide this to free its own Box<str>
}

namespace DroxBridge {

// Initializer: Returns 0 on success
int init_orchestrator() {
    // Rust should return an error code instead of panicking
    return rust_init_orchestrator();
}

// Wrapper for running the swarm
// Returns a JSON string (caller is responsible for calling free_string)

char* run_swarm_ffi(const char* prompt, const std::vector<const char*>& files) {
    try {
        // Files.data() returns const char* const*, but Rust's extern "C" 
        // usually expects const char**. We cast the top-level pointer.
        return rust_run_swarm(
            prompt, 
            const_cast<const char**>(files.data()), 
            static_cast<int>(files.size())
        );
    } catch (const std::exception& e) {
        fprintf(stderr, "Swarm execution failed: %s\n", e.what());
        return nullptr;
    }
}

// HITL: Accept a suggested change
void accept_diff(const std::string& diff_id) {
    if (diff_id.empty()) return;
    rust_accept_diff(diff_id.c_str());
}

// HITL: Reject with feedback for the Janitor agent to learn
void reject_diff(const std::string& diff_id, const std::string& feedback) {
    if (diff_id.empty()) return;
    rust_reject_diff(diff_id.c_str(), feedback.c_str());
}

// Metrics Retrieval
char* get_metrics() {
    return rust_get_metrics_summary();
}

// CRITICAL: Memory safety helper
// Rust strings allocated with CString::into_raw must be freed by Rust's allocator
void free_rust_string(char* str) {
    if (str) {
        rust_free_string(str);
    }
}

} // namespace DroxBridge