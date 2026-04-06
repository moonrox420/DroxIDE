// Minimal C++ test harness - links to Rust core and runs a swarm workflow
#include <iostream>
#include <string>

// Rust FFI functions
extern "C" {
    int init_orchestrator();
    char* run_swarm(const char* prompt, const char** context_files, int file_count);
    void accept_diff(const char* diff_id);
    void reject_diff(const char* diff_id, const char* feedback);
    char* get_orchestrator_state();
    char* get_metrics_summary();
    void free_string(char* str);
}

int main() {
    std::cout << "========================================\n";
    std::cout << "DroxIDE Rust Core - Integration Test\n";
    std::cout << "========================================\n\n";

    // Initialize orchestrator
    std::cout << "[1/4] Initializing orchestrator...\n";
    init_orchestrator();
    std::cout << "      SUCCESS\n\n";

    // Get state
    std::cout << "[2/4] Getting orchestrator state...\n";
    char* state = get_orchestrator_state();
    if (state) {
        std::cout << "      State: " << state << "\n";
        free_string(state);
    }
    std::cout << "\n";

    // Run swarm
    std::cout << "[3/4] Running swarm workflow...\n";
    std::cout << "      Prompt: \"Add error handling to main.rs\"\n";
    const char* files[] = { "src-rust/main.rs" };
    char* result = run_swarm("Add error handling to main.rs", files, 1);
    if (result) {
        std::cout << "      Result: " << result << "\n";
        free_string(result);
    }
    std::cout << "\n";

    // Get metrics
    std::cout << "[4/4] Getting metrics...\n";
    char* metrics = get_metrics_summary();
    if (metrics) {
        std::cout << "      Metrics: " << metrics << "\n";
        free_string(metrics);
    }
    std::cout << "\n";

    std::cout << "========================================\n";
    std::cout << "ALL RUST CORE FUNCTIONS EXECUTED\n";
    std::cout << "========================================\n";
    return 0;
}
