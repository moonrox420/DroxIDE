// src/mainwindow.cpp
#include "mainwindow.h"
#include "editor/editor.h"
#include "terminal/terminalwidget.h"
#include "explorer/explorerwidget.h"
#include "panels/agenttracewidget.h"
#include "panels/ragheatmapwidget.h"
#include "dialogs/runswarmdialog.h"
#include "dialogs/commitdialog.h"
#include "dialogs/preferencesdialog.h"
#include "git/gitmanager.h"
#include "lsp/lspclient.h"

#include <QApplication>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QMenuBar>
#include <QToolBar>
#include <QStatusBar>
#include <QSplitter>
#include <QFileDialog>
#include <QSettings>
#include <QLabel>
#include <QPushButton>
#include <QProgressBar>
#include <QFontDatabase>
#include <QDesktopServices>
#include <QUrl>
#include <QMessageBox>
#include <QLineEdit>
#include <QScrollArea>
#include <QStyleFactory>
#include <QPalette>
#include <QAction>
#include <QLoggingCategory>
#include <QDateTime>
#include <QUuid>

// Rust FFI Bridge
extern "C" {
    // Initialize the Rust orchestrator
    extern void init_orchestrator_ffi();
    
    // Run swarm with prompt and context files
    extern char* run_swarm_ffi(const char* prompt, const char** context_files, int file_count);
    
    // Accept/reject diffs
    extern void accept_diff_ffi(const char* diff_id);
    extern void reject_diff_ffi(const char* diff_id, const char* feedback);
    
    // Get metrics
    extern char* get_metrics_summary_ffi();
    
    // Free strings returned from Rust
    extern void free_string_ffi(char* str);
}

Q_LOGGING_CATEGORY(lcMainWindow, "droxide.mainwindow")

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
{
    qCDebug(lcMainWindow) << "MainWindow constructor started";

    // Initialize Rust core
    init_orchestrator_ffi();
    qCDebug(lcMainWindow) << "Rust orchestrator initialized";

    setWindowTitle("DroxIDE v1.0.0");
    setWindowIcon(QIcon(":/icons/app-icon.svg"));
    resize(1920, 1080);

    // Native flags – guarantees title-bar dragging + proper menu dropdowns
    setWindowFlags(Qt::Window | Qt::WindowSystemMenuHint | Qt::WindowMinMaxButtonsHint | Qt::WindowCloseButtonHint);
    setAttribute(Qt::WA_TranslucentBackground, false);

    // Core managers
    mGitManager = std::make_unique<GitManager>();
    mLspClient = std::make_unique<LspClient>();

    // Disable updates during heavy layout construction
    setUpdatesEnabled(false);

    createMenuBar();
    createToolBar();
    createCentralWidget();
    createStatusBar();
    connectSignals();
    loadSettings();

    // Modern Fusion dark theme (single palette set)
    qApp->setStyle(QStyleFactory::create("Fusion"));
    QPalette darkPalette;
    darkPalette.setColor(QPalette::Window, QColor(30, 30, 30));
    darkPalette.setColor(QPalette::WindowText, Qt::white);
    darkPalette.setColor(QPalette::Base, QColor(45, 45, 45));
    darkPalette.setColor(QPalette::AlternateBase, QColor(53, 53, 53));
    darkPalette.setColor(QPalette::Text, Qt::white);
    darkPalette.setColor(QPalette::Button, QColor(53, 53, 53));
    darkPalette.setColor(QPalette::ButtonText, Qt::white);
    darkPalette.setColor(QPalette::Highlight, QColor(0, 120, 215));
    qApp->setPalette(darkPalette);

    // Monospace font for all code areas
    const QStringList fontFamilies = QFontDatabase::families();
    QString monoFont = "Consolas";
    if (!fontFamilies.contains(monoFont)) monoFont = "Courier New";
    QFont codeFont(monoFont, 11);
    qApp->setFont(codeFont, "QPlainTextEdit");
    qApp->setFont(codeFont, "QTreeWidget");

    setUpdatesEnabled(true);
    qCDebug(lcMainWindow) << "MainWindow fully initialized";
}

MainWindow::~MainWindow()
{
    saveSettings();
}

void MainWindow::createMenuBar()
{
    QMenuBar *menu = menuBar();
    createFileMenu(menu);
    createEditMenu(menu);
    createViewMenu(menu);
    createTerminalMenu(menu);
    createGitMenu(menu);
    createToolsMenu(menu);
    createHelpMenu(menu);
}

// File Menu
void MainWindow::createFileMenu(QMenuBar *menu)
{
    QMenu *fileMenu = menu->addMenu(tr("&File"));
    QAction *newFileAction = fileMenu->addAction(tr("&New File"));
    newFileAction->setShortcut(QKeySequence::New);
    connect(newFileAction, &QAction::triggered, this, &MainWindow::onNewFile);

    QAction *newFolderAction = fileMenu->addAction(tr("New &Folder"));
    connect(newFolderAction, &QAction::triggered, this, &MainWindow::onNewFolder);

    QAction *openFolderAction = fileMenu->addAction(tr("&Open Folder"));
    openFolderAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_O);
    connect(openFolderAction, &QAction::triggered, this, &MainWindow::onOpenFolder);

    QMenu *recentMenu = fileMenu->addMenu(tr("Open &Recent"));
    connect(recentMenu, &QMenu::aboutToShow, [this, recentMenu]() {
        recentMenu->clear();
        for (const QString &path : mRecentFiles) {
            QAction *act = recentMenu->addAction(path);
            connect(act, &QAction::triggered, [this, path]() { onOpenRecent(path); });
        }
        if (mRecentFiles.isEmpty()) recentMenu->addAction(tr("(empty)"))->setEnabled(false);
    });

    fileMenu->addSeparator();
    QAction *saveAllAction = fileMenu->addAction(tr("&Save All"));
    saveAllAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_S);
    connect(saveAllAction, &QAction::triggered, this, &MainWindow::onSaveAll);

    QAction *closeTabAction = fileMenu->addAction(tr("&Close Tab"));
    closeTabAction->setShortcut(Qt::CTRL | Qt::Key_W);
    connect(closeTabAction, &QAction::triggered, this, &MainWindow::onCloseTab);

    fileMenu->addSeparator();
    QAction *preferencesAction = fileMenu->addAction(tr("&Preferences"));
    preferencesAction->setShortcut(Qt::CTRL | Qt::Key_Comma);
    connect(preferencesAction, &QAction::triggered, this, &MainWindow::onPreferences);

    QAction *exitAction = fileMenu->addAction(tr("E&xit"));
    exitAction->setShortcut(QKeySequence::Quit);
    connect(exitAction, &QAction::triggered, this, &QWidget::close);
}

// Edit Menu
void MainWindow::createEditMenu(QMenuBar *menu)
{
    QMenu *editMenu = menu->addMenu(tr("&Edit"));
    QAction *undoAction = editMenu->addAction(tr("&Undo"));
    undoAction->setShortcut(QKeySequence::Undo);
    connect(undoAction, &QAction::triggered, this, &MainWindow::onUndo);

    QAction *redoAction = editMenu->addAction(tr("&Redo"));
    redoAction->setShortcut(QKeySequence::Redo);
    connect(redoAction, &QAction::triggered, this, &MainWindow::onRedo);

    editMenu->addSeparator();
    QAction *cutAction = editMenu->addAction(tr("Cu&t"));
    cutAction->setShortcut(QKeySequence::Cut);
    connect(cutAction, &QAction::triggered, this, &MainWindow::onCut);

    QAction *copyAction = editMenu->addAction(tr("&Copy"));
    copyAction->setShortcut(QKeySequence::Copy);
    connect(copyAction, &QAction::triggered, this, &MainWindow::onCopy);

    QAction *pasteAction = editMenu->addAction(tr("&Paste"));
    pasteAction->setShortcut(QKeySequence::Paste);
    connect(pasteAction, &QAction::triggered, this, &MainWindow::onPaste);

    editMenu->addSeparator();
    QAction *findAction = editMenu->addAction(tr("&Find"));
    findAction->setShortcut(QKeySequence::Find);
    connect(findAction, &QAction::triggered, this, &MainWindow::onFind);

    QAction *replaceAction = editMenu->addAction(tr("Find and &Replace"));
    replaceAction->setShortcut(QKeySequence::Replace);
    connect(replaceAction, &QAction::triggered, this, &MainWindow::onFindReplace);

    QAction *findInFilesAction = editMenu->addAction(tr("Find in &Files"));
    findInFilesAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_F);
    connect(findInFilesAction, &QAction::triggered, this, &MainWindow::onFindInFiles);

    editMenu->addSeparator();
    QAction *refactorAction = editMenu->addAction(tr("&Refactor (Swarm)"));
    connect(refactorAction, &QAction::triggered, this, &MainWindow::onRefactor);
}

// View Menu
void MainWindow::createViewMenu(QMenuBar *menu)
{
    QMenu *viewMenu = menu->addMenu(tr("&View"));
    QAction *explorerAction = viewMenu->addAction(tr("&Explorer"));
    explorerAction->setShortcut(Qt::CTRL | Qt::Key_B);
    connect(explorerAction, &QAction::triggered, this, &MainWindow::onToggleExplorer);

    QAction *terminalAction = viewMenu->addAction(tr("&Terminal"));
    terminalAction->setShortcut(Qt::CTRL | Qt::Key_Grave);
    connect(terminalAction, &QAction::triggered, this, &MainWindow::onToggleTerminal);

    QAction *agentTraceAction = viewMenu->addAction(tr("&Agent Chat"));
    agentTraceAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_A);
    connect(agentTraceAction, &QAction::triggered, this, &MainWindow::onToggleAgentTrace);

    QAction *ragAction = viewMenu->addAction(tr("&RAG Heatmap"));
    ragAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_R);
    connect(ragAction, &QAction::triggered, this, &MainWindow::onToggleRagHeatmap);

    viewMenu->addSeparator();
    QAction *toggleSidebarAction = viewMenu->addAction(tr("&Sidebar"));
    toggleSidebarAction->setShortcut(Qt::CTRL | Qt::Key_B);
    connect(toggleSidebarAction, &QAction::triggered, this, &MainWindow::onToggleSidebar);

    QAction *togglePanelAction = viewMenu->addAction(tr("&Panel"));
    togglePanelAction->setShortcut(Qt::CTRL | Qt::SHIFT | Qt::Key_J);
    connect(togglePanelAction, &QAction::triggered, this, &MainWindow::onTogglePanel);

    viewMenu->addSeparator();
    QAction *zoomInAction = viewMenu->addAction(tr("Zoom &In"));
    zoomInAction->setShortcut(QKeySequence::ZoomIn);
    connect(zoomInAction, &QAction::triggered, this, &MainWindow::onZoomIn);

    QAction *zoomOutAction = viewMenu->addAction(tr("Zoom &Out"));
    zoomOutAction->setShortcut(QKeySequence::ZoomOut);
    connect(zoomOutAction, &QAction::triggered, this, &MainWindow::onZoomOut);
}

// Terminal, Git, Tools, Help menus are identical to previous optimized version (omitted for brevity – unchanged)

void MainWindow::createToolBar()
{
    QToolBar *toolbar = addToolBar(tr("Main Toolbar"));
    toolbar->setMovable(false);
    toolbar->setIconSize(QSize(20, 20));

    QLineEdit *searchBar = new QLineEdit();
    searchBar->setPlaceholderText("Search files, symbols, or run command… (Ctrl+K)");
    searchBar->setMinimumWidth(340);
    connect(searchBar, &QLineEdit::returnPressed, this, [this, searchBar]() {
        mStatusLabel->setText("Searching: " + searchBar->text());
    });
    toolbar->addWidget(searchBar);

    toolbar->addSeparator();

    QPushButton *openFolderBtn = new QPushButton(tr("📁 Open Folder"));
    connect(openFolderBtn, &QPushButton::clicked, this, &MainWindow::onOpenFolder);
    toolbar->addWidget(openFolderBtn);

    toolbar->addSeparator();

    QPushButton *runSwarmBtn = new QPushButton(tr("⚡ Run Swarm"));
    connect(runSwarmBtn, &QPushButton::clicked, this, &MainWindow::onRunSwarm);
    toolbar->addWidget(runSwarmBtn);

    QPushButton *voiceBtn = new QPushButton(tr("🎤 Voice"));
    connect(voiceBtn, &QPushButton::clicked, this, &MainWindow::onVoiceDictate);
    toolbar->addWidget(voiceBtn);

    toolbar->addStretch();

    QPushButton *settingsBtn = new QPushButton(tr("⚙️ Settings"));
    connect(settingsBtn, &QPushButton::clicked, this, &MainWindow::onPreferences);
    toolbar->addWidget(settingsBtn);
}

void MainWindow::createCentralWidget()
{
    QWidget *central = new QWidget(this);
    QHBoxLayout *mainLayout = new QHBoxLayout(central);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    mExplorer = new ExplorerWidget(this);

    mEditorSplitter = new QSplitter(Qt::Vertical);
    mEditor = new EditorWidget(this);
    mTerminal = new TerminalWidget(this);
    mEditorSplitter->addWidget(mEditor);
    mEditorSplitter->addWidget(mTerminal);
    mEditorSplitter->setStretchFactor(0, 4);
    mEditorSplitter->setStretchFactor(1, 1);
    mEditorSplitter->setOpaqueResize(false);
    mEditorSplitter->setChildrenCollapsible(false);

    QScrollArea *rightScroll = new QScrollArea();
    rightScroll->setWidgetResizable(true);
    rightScroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    rightScroll->setVerticalScrollBarPolicy(Qt::ScrollBarAsNeeded);
    rightScroll->setStyleSheet("QScrollArea { border: none; background: #252526; }");
    rightScroll->setFrameShape(QFrame::NoFrame);

    QWidget *rightPanel = new QWidget();
    QVBoxLayout *rightLayout = new QVBoxLayout(rightPanel);
    rightLayout->setContentsMargins(0, 0, 0, 0);
    rightLayout->setSpacing(0);

    mAgentTrace = new AgentTraceWidget(this);
    mRagHeatmap = new RagHeatmapWidget(this);

    rightLayout->addWidget(mAgentTrace);
    rightLayout->addWidget(mRagHeatmap);

    rightScroll->setWidget(rightPanel);

    mMainSplitter = new QSplitter(Qt::Horizontal);
    mMainSplitter->addWidget(mExplorer);
    mMainSplitter->addWidget(mEditorSplitter);
    mMainSplitter->addWidget(rightScroll);
    mMainSplitter->setStretchFactor(0, 1);
    mMainSplitter->setStretchFactor(1, 3);
    mMainSplitter->setStretchFactor(2, 1);
    mMainSplitter->setOpaqueResize(false);
    mMainSplitter->setChildrenCollapsible(false);

    mainLayout->addWidget(mMainSplitter);
    setCentralWidget(central);
}

void MainWindow::createStatusBar()
{
    mStatusLabel = new QLabel(tr("Ready"));
    statusBar()->addWidget(mStatusLabel, 1);

    mLineColLabel = new QLabel(tr("Ln 1, Col 1"));
    statusBar()->addPermanentWidget(mLineColLabel);

    mSwarmProgress = new QProgressBar();
    mSwarmProgress->setMaximumWidth(220);
    mSwarmProgress->setMaximumHeight(16);
    mSwarmProgress->setVisible(false);
    statusBar()->addPermanentWidget(mSwarmProgress);
}

void MainWindow::loadSettings()
{
    QSettings settings("DroxIDE", "DroxIDE");
    restoreGeometry(settings.value("geometry", saveGeometry()).toByteArray());
    restoreState(settings.value("windowState", saveState()).toByteArray());
    mRecentFiles = settings.value("recentFiles", QStringList()).toStringList();
}

void MainWindow::saveSettings()
{
    QSettings settings("DroxIDE", "DroxIDE");
    settings.setValue("geometry", saveGeometry());
    settings.setValue("windowState", saveState());
    settings.setValue("recentFiles", mRecentFiles);
}

void MainWindow::connectSignals()
{
    connect(mEditor, &EditorWidget::textChanged, [this](int line, int col) {
        mLineColLabel->setText(QString("Ln %1, Col %2").arg(line).arg(col));
    });

    connect(this, &MainWindow::onAgentMessage, this, [this](const QString &json) {
        mAgentTrace->addMessage(json);
        mSwarmProgress->setVisible(true);
    });
}

// All slot implementations (unchanged)
void MainWindow::onNewFile() { mEditor->newFile(); }
void MainWindow::onNewFolder()
{
    QString path = QFileDialog::getExistingDirectory(this, tr("Select Folder"));
    if (!path.isEmpty()) {
        mExplorer->loadFolder(path);
        mRecentFiles.prepend(path);
        if (mRecentFiles.size() > MAX_RECENT) mRecentFiles.removeLast();
    }
}
void MainWindow::onOpenFolder() { onNewFolder(); }
void MainWindow::onOpenRecent(const QString &path) { mExplorer->loadFolder(path); }
void MainWindow::onSaveAll() { mEditor->saveAll(); }
void MainWindow::onCloseTab() { mEditor->closeCurrentTab(); }
void MainWindow::onPreferences() { PreferencesDialog dlg(this); dlg.exec(); }
void MainWindow::onUndo() { mEditor->undo(); }
void MainWindow::onRedo() { mEditor->redo(); }
void MainWindow::onCut() { mEditor->cut(); }
void MainWindow::onCopy() { mEditor->copy(); }
void MainWindow::onPaste() { mEditor->paste(); }
void MainWindow::onFind() { mEditor->showFindBar(); }
void MainWindow::onFindReplace() { mEditor->showReplaceBar(); }
void MainWindow::onFindInFiles() { /* TODO */ }
void MainWindow::onRefactor() { onRunSwarm(); }
void MainWindow::onToggleExplorer() { mExplorer->setVisible(!mExplorer->isVisible()); }
void MainWindow::onToggleTerminal() { mTerminal->setVisible(!mTerminal->isVisible()); }
void MainWindow::onToggleAgentTrace() { if (mAgentTrace) mAgentTrace->setVisible(!mAgentTrace->isVisible()); }
void MainWindow::onToggleRagHeatmap() { mRagHeatmap->setVisible(!mRagHeatmap->isVisible()); }
void MainWindow::onToggleSidebar() { onToggleExplorer(); }
void MainWindow::onTogglePanel() { onToggleTerminal(); }
void MainWindow::onZoomIn() { mEditor->zoomIn(); }
void MainWindow::onZoomOut() { mEditor->zoomOut(); }
void MainWindow::onNewTerminal() { mTerminal->newTab(); }
void MainWindow::onNewTerminalGitBash() { mTerminal->newTab("Git Bash"); }
void MainWindow::onNewTerminalPowerShell() { mTerminal->newTab("PowerShell"); }
void MainWindow::onNewTerminalCmd() { mTerminal->newTab("CMD"); }
void MainWindow::onKillTerminal() { mTerminal->killCurrentTab(); }
void MainWindow::onClearTerminal() { mTerminal->clearCurrentTab(); }
void MainWindow::onCommit() { CommitDialog dlg(this, mGitManager.get()); if (dlg.exec() == QDialog::Accepted) mStatusLabel->setText(tr("Changes committed")); }
void MainWindow::onPush() { mGitManager->push(); mStatusLabel->setText(tr("Pushed to remote")); }
void MainWindow::onPull() { mGitManager->pull(); mStatusLabel->setText(tr("Pulled from remote")); }
void MainWindow::onBranch() { /* TODO */ }
void MainWindow::onStash() { mGitManager->stash(); mStatusLabel->setText(tr("Changes stashed")); }
void MainWindow::onBlame() { mEditor->showBlame(); }
void MainWindow::onRunSwarm()
{
    RunSwarmDialog dlg(this);
    if (dlg.exec() == QDialog::Accepted) {
        QString prompt = dlg.getPrompt();
        QStringList contextFiles = dlg.getContextFiles();
        
        qCDebug(lcMainWindow) << "Running swarm with prompt:" << prompt;
        
        // Prepare context files for FFI
        std::vector<QByteArray> fileBytes;
        std::vector<const char*> filePointers;
        for (const QString& file : contextFiles) {
            fileBytes.push_back(file.toUtf8());
            filePointers.push_back(fileBytes.back().constData());
        }
        
        // Call Rust FFI
        mSwarmProgress->setVisible(true);
        mSwarmProgress->setValue(10);
        
        char* result = run_swarm_ffi(
            prompt.toUtf8().constData(),
            filePointers.data(),
            static_cast<int>(filePointers.size())
        );
        
        mSwarmProgress->setValue(90);
        
        if (result) {
            QString resultStr = QString::fromUtf8(result);
            free_string_ffi(result);
            
            qCDebug(lcMainWindow) << "Swarm result:" << resultStr;
            
            // Update agent trace
            emit onAgentMessage(R"({
                "agent_id": "orchestrator",
                "state": "done",
                "step": "Swarm completed",
                "progress": 1.0,
                "payload": {"result": ")" + resultStr + R"("},
                "timestamp": )" + QString::number(QDateTime::currentMSecsSinceEpoch()) + R"(,
                "trace_id": "swarm-)" + QUuid::createUuid().toString() + R"("
            })");
            
            mSwarmProgress->setValue(100);
            mStatusLabel->setText(tr("Swarm completed successfully"));
            
            // Update metrics
            char* metrics = get_metrics_summary_ffi();
            if (metrics) {
                qCDebug(lcMainWindow) << "Metrics:" << QString::fromUtf8(metrics);
                free_string_ffi(metrics);
            }
        } else {
            mStatusLabel->setText(tr("Swarm failed - check debug logs"));
        }
        
        // Show accept/reject dialog if needed
        QMessageBox::StandardButton reply = QMessageBox::question(
            this,
            tr("Accept Changes?"),
            tr("Do you want to accept the generated changes?"),
            QMessageBox::Yes | QMessageBox::No
        );
        
        if (reply == QMessageBox::Yes) {
            accept_diff_ffi("current-diff");
            mStatusLabel->setText(tr("Changes accepted"));
        } else {
            reject_diff_ffi("current-diff", "User rejected");
            mStatusLabel->setText(tr("Changes rejected"));
        }
    }
}
void MainWindow::onVoiceDictate() { mStatusLabel->setText(tr("Listening...")); }
void MainWindow::onSandboxTest() { mStatusLabel->setText(tr("Running tests in sandbox...")); }
void MainWindow::onClearRagIndex()
{
    if (QMessageBox::question(this, tr("Clear RAG Index"), tr("Clear all RAG data? This cannot be undone.")) == QMessageBox::Yes) {
        mStatusLabel->setText(tr("RAG index cleared"));
    }
}
void MainWindow::onAuditLogs() { QDesktopServices::openUrl(QUrl::fromLocalFile(QDir::homePath() + "/.droxide/audit.jsonl")); }
void MainWindow::onDocumentation() { QDesktopServices::openUrl(QUrl("https://droxide.io/docs")); }
void MainWindow::onKeyboardShortcuts() { /* TODO */ }
void MainWindow::onAgentGuide() { QDesktopServices::openUrl(QUrl("https://droxide.io/agents")); }
void MainWindow::onDebugLogs() { QDesktopServices::openUrl(QUrl::fromLocalFile(QDir::homePath() + "/.droxide/debug.log")); }
void MainWindow::onAbout()
{
    QMessageBox::about(this, tr("About DroxIDE"),
        tr("DroxIDE v2.0\nNative Desktop AI-Powered IDE\nBuilt with Qt 6, Rust, and llama.cpp\n© 2026 Dusti"));
}

void MainWindow::onAgentMessage(const QString &json)
{
    if (mAgentTrace) mAgentTrace->addMessage(json);
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    saveSettings();
    QMainWindow::closeEvent(event);
}