#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QSplitter>
#include <QLabel>
#include <QProgressBar>
#include <QStringList>
#include <memory>

// Forward declarations to keep compile times lean
class Editor;
class TerminalWidget;
class ExplorerWidget;
class AgentTraceWidget;
class RagHeatmapWidget;
class GitManager;
class LspClient;
class SwarmBridge; // The FFI Bridge to Rust

/**
 * @brief DroxIDE Main Window
 * Coordinates the Qt 6 frontend with the Rust-based Agent Swarm.
 */
class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    explicit MainWindow(QWidget *parent = nullptr);
    ~MainWindow() override;

protected:
    void closeEvent(QCloseEvent *event) override;

private slots:
    // --- File & Workspace ---
    void onNewFile();
    void onNewFolder();
    void onOpenFolder();
    void onOpenRecent(const QString &path);
    void onSaveAll();
    void onCloseTab();
    void onPreferences();

    // --- View Management ---
    void onToggleExplorer();
    void onToggleTerminal();
    void onToggleAgentTrace();
    void onToggleRagHeatmap();
    void onZoomIn();
    void onZoomOut();

    // --- Edit Operations ---
    void onUndo();
    void onRedo();
    void onCut();
    void onCopy();
    void onPaste();
    void onFind();
    void onFindReplace();
    void onFindInFiles();
    void onRefactor();

    // --- Terminal Operations ---
    void onNewTerminal();
    void onNewTerminalGitBash();
    void onNewTerminalPowerShell();
    void onNewTerminalCmd();
    void onKillTerminal();
    void onClearTerminal();

    // --- Git Operations ---
    void onStash();
    void onBranch();

    // --- AI Swarm Operations ---
    void onRunSwarm();
    void onVoiceDictate();
    void onSandboxTest();
    void onClearRagIndex();
    
    // --- Git Operations ---
    void onCommit();
    void onPush();
    void onPull();
    void onBlame();

    // --- Bridge Callbacks (Rust -> C++) ---
    // These should be invoked via Qt::QueuedConnection for thread safety
    void onAgentMessage(const QString &json);
    void onSwarmStateChanged(const QString &state);
    void onUserActionRequired(const QString &payload);

    // --- Help & Logs ---
    void onDocumentation();
    void onKeyboardShortcuts();
    void onAgentGuide();
    void onDebugLogs();
    void onAbout();

private:
    // Initialization
    void setupBranding();
    void initUi();
    void applyTheme();
    void createMenuBar();
    void createToolBar();
    void createCentralWidget();
    void createStatusBar();
    void connectSignals();
    
    // Persistence
    void loadSettings();
    void saveSettings();
    void updateRecentFiles(const QString &path);

    // --- UI Components (RAII managed by Qt parent system) ---
    Editor* mEditor       = nullptr;
    TerminalWidget* mTerminal     = nullptr;
    ExplorerWidget* mExplorer     = nullptr;
    AgentTraceWidget* mAgentTrace   = nullptr;
    RagHeatmapWidget* mRagHeatmap   = nullptr;

    // --- Layout Components ---
    QSplitter* mMainSplitter   = nullptr; // [Explorer | Editor/Terminal | Trace/RAG]
    QSplitter* mEditorSplitter = nullptr; // [Editor | Terminal]

    // --- Status Bar Widgets ---
    QLabel* mStatusLabel    = nullptr;
    QLabel* mLineColLabel   = nullptr;
    QProgressBar* mSwarmProgress  = nullptr;

    // --- Backend Managers (RAII managed by std::unique_ptr) ---
    std::unique_ptr<GitManager>  mGitManager;
    std::unique_ptr<LspClient>   mLspClient;
    std::unique_ptr<SwarmBridge> mSwarmBridge; // Handles Rust orchestration

    // --- State ---
    QStringList mRecentFiles;
    static constexpr int MAX_RECENT = 10;
};

#endif // MAINWINDOW_H