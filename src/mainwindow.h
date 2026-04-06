// src/mainwindow.h
#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QSplitter>
#include <QStackedWidget>
#include <QLabel>
#include <QProgressBar>
#include <memory>

class EditorWidget;
class TerminalWidget;
class ExplorerWidget;
class AgentTraceWidget;
class RagHeatmapWidget;
class GitManager;
class LspClient;

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

protected:
    void closeEvent(QCloseEvent *event) override;

private slots:
    // File menu
    void onNewFile();
    void onNewFolder();
    void onOpenFolder();
    void onOpenRecent(const QString &path);
    void onSaveAll();
    void onCloseTab();
    void onPreferences();

    // Edit menu
    void onUndo();
    void onRedo();
    void onCut();
    void onCopy();
    void onPaste();
    void onFind();
    void onFindReplace();
    void onFindInFiles();
    void onRefactor();

    // View menu
    void onToggleExplorer();
    void onToggleTerminal();
    void onToggleAgentTrace();
    void onToggleRagHeatmap();
    void onToggleSidebar();
    void onTogglePanel();
    void onZoomIn();
    void onZoomOut();

    // Terminal menu
    void onNewTerminal();
    void onNewTerminalGitBash();
    void onNewTerminalPowerShell();
    void onNewTerminalCmd();
    void onKillTerminal();
    void onClearTerminal();

    // Git menu
    void onCommit();
    void onPush();
    void onPull();
    void onBranch();
    void onStash();
    void onBlame();

    // Tools menu
    void onRunSwarm();
    void onVoiceDictate();
    void onSandboxTest();
    void onClearRagIndex();
    void onAuditLogs();

    // Help menu
    void onDocumentation();
    void onKeyboardShortcuts();
    void onAgentGuide();
    void onDebugLogs();
    void onAbout();

    // Agent events from Rust
    void onAgentMessage(const QString &json);
    void onSwarmStateChanged(const QString &state);
    void onUserActionRequired(const QString &payload);

private:
    void createMenuBar();
    void createToolBar();
    void createCentralWidget();
    void createDockWidgets();
    void createStatusBar();
    void loadSettings();
    void saveSettings();
    void connectSignals();

    // UI Components
    EditorWidget *mEditor = nullptr;
    TerminalWidget *mTerminal = nullptr;
    ExplorerWidget *mExplorer = nullptr;
    AgentTraceWidget *mAgentTrace = nullptr;
    RagHeatmapWidget *mRagHeatmap = nullptr;

    // Core managers
    std::unique_ptr<GitManager> mGitManager;
    std::unique_ptr<LspClient> mLspClient;

    // Layout
    QSplitter *mMainSplitter = nullptr;
    QSplitter *mEditorSplitter = nullptr;
    QStackedWidget *mViewStack = nullptr;

    // Status bar
    QLabel *mStatusLabel = nullptr;
    QLabel *mLineColLabel = nullptr;
    QProgressBar *mSwarmProgress = nullptr;

    // Recent files
    QStringList mRecentFiles;
    static const int MAX_RECENT = 10;
};

#endif // MAINWINDOW_H
