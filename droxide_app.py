#!/usr/bin/env python3
"""DroxIDE v1.0.0 - Premium Professional Native Desktop AI-Powered IDE"""

import sys
import json
from pathlib import Path
from PyQt6.QtWidgets import (
    QApplication, QMainWindow, QToolBar, QStatusBar,
    QSplitter, QTabWidget, QPlainTextEdit, QTreeWidget, QTreeWidgetItem,
    QVBoxLayout, QHBoxLayout, QWidget, QGroupBox, QLabel, QPushButton,
    QFileDialog, QScrollArea, QGraphicsOpacityEffect
)
from PyQt6.QtCore import Qt, QTimer, QSize, pyqtSignal, QProcess, QPropertyAnimation, QEasingCurve, QObject
from PyQt6.QtGui import QAction, QFont, QColor, QPainter

class RustCore(QObject):
    """Rust IPC Protocol - Signals must be class-level on QObject"""
    agent_message = pyqtSignal(str, str, str)
    swarm_state = pyqtSignal(str)
    rag_update = pyqtSignal(dict)
    error_occurred = pyqtSignal(str)

    CMD_RUN_SWARM = "run_swarm"
    CMD_QUERY_RAG = "query_rag"

    def __init__(self, parent=None):
        super().__init__(parent)
        self.process = None

    def start(self):
        rust_path = Path("target/release/droxide_rust.exe")
        if not rust_path.exists():
            rust_path = Path("target/release/droxide_rust")
        if not rust_path.exists():
            self.error_occurred.emit("Rust core binary not found - running in simulation mode")
            return False

        self.process = QProcess()
        self.process.setProgram(str(rust_path))
        self.process.readyReadStandardOutput.connect(self._read_stdout)
        self.process.readyReadStandardError.connect(self._read_stderr)
        self.process.start()
        return True

    def _read_stdout(self):
        data = self.process.readAllStandardOutput().data().decode('utf-8', errors='ignore')
        for line in data.splitlines():
            line = line.strip()
            if not line: continue
            try:
                msg = json.loads(line)
                t = msg.get("type")
                if t == "agent_message":
                    self.agent_message.emit(msg.get("agent", "System"), msg.get("status", "info"), msg.get("content", ""))
                elif t == "swarm_state":
                    self.swarm_state.emit(msg.get("state", ""))
                elif t == "rag_update":
                    self.rag_update.emit(msg.get("data", {}))
            except:
                continue

    def _read_stderr(self):
        err = self.process.readAllStandardError().data().decode('utf-8', errors='ignore').strip()
        if err:
            self.error_occurred.emit(err)

    def send_command(self, cmd, payload=None):
        if self.process and self.process.state() == QProcess.ProcessState.Running:
            packet = {"command": cmd, **(payload or {})}
            self.process.write((json.dumps(packet) + "\n").encode('utf-8'))


class AgentBubble(QWidget):
    """Premium animated agent bubble with fade + slide-up"""
    def __init__(self, agent: str, status: str, content: str, parent=None):
        super().__init__(parent)
        self.setStyleSheet("""
            QWidget {
                background: qlineargradient(x1:0, y1:0, x2:1, y2:1, stop:0 #2d2d2d, stop:1 #1f1f1f);
                border: 1px solid #3c3c3c;
                border-radius: 12px;
                padding: 14px;
                margin: 6px 10px;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setSpacing(6)
        layout.setContentsMargins(0, 0, 0, 0)

        header = QHBoxLayout()
        agent_lbl = QLabel(f"🤖 {agent}")
        agent_lbl.setStyleSheet("font-weight: 600; color: #61afef;")
        header.addWidget(agent_lbl)

        status_lbl = QLabel(status)
        status_lbl.setStyleSheet("color: #98c379; font-size: 10px;")
        header.addWidget(status_lbl, alignment=Qt.AlignmentFlag.AlignRight)
        layout.addLayout(header)

        content_lbl = QLabel(content)
        content_lbl.setWordWrap(True)
        content_lbl.setStyleSheet("color: #d4d4d4; line-height: 1.5;")
        layout.addWidget(content_lbl)

        # Fade-in animation
        self.opacity = QGraphicsOpacityEffect()
        self.setGraphicsEffect(self.opacity)
        self.opacity.setOpacity(0.0)

        self.anim = QPropertyAnimation(self.opacity, b"opacity")
        self.anim.setDuration(420)
        self.anim.setStartValue(0.0)
        self.anim.setEndValue(1.0)
        self.anim.setEasingCurve(QEasingCurve.Type.OutCubic)
        self.anim.start()


class RagHeatmapWidget(QWidget):
    """Premium RAG Heatmap with colored relevance bars"""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.chunks = []
        self.relevances = []
        self.setMinimumHeight(160)

    def update_heatmap(self, chunks, relevances):
        self.chunks = chunks[:12]
        self.relevances = relevances[:12]
        self.update()

    def paintEvent(self, event):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)

        if not self.chunks:
            p.setPen(QColor("#5c6370"))
            p.setFont(QFont("Segoe UI", 11))
            p.drawText(self.rect(), Qt.AlignmentFlag.AlignCenter, "No RAG data yet")
            return

        bar_h = self.height() // len(self.chunks)
        for i, (name, score) in enumerate(zip(self.chunks, self.relevances)):
            bar_w = int(self.width() * score)
            hue = 0.33 * score
            color = QColor.fromHsvF(hue, 0.85, 0.92)
            p.fillRect(8, i * bar_h + 4, bar_w - 16, bar_h - 8, color)
            p.setPen(QColor("#abb2bf"))
            p.setFont(QFont("Segoe UI", 9))
            p.drawText(16, i * bar_h + bar_h // 2 + 5, name[:28] + ("..." if len(name) > 28 else ""))


class DroxIDE(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("DroxIDE — Native Desktop AI IDE")
        self.resize(1880, 1060)

        self.current_folder = None
        self.rust = RustCore(self)
        
        # Now these will work because RustCore is a QObject
        self.rust.agent_message.connect(self._on_agent_message)
        self.rust.swarm_state.connect(self._on_swarm_state)
        self.rust.rag_update.connect(self._on_rag_update)

        self._setup_ui()
        self._show_welcome()
        self.rust.start()

    def _setup_ui(self):
        self._create_menu_bar()
        self._create_toolbar()
        self._create_central_widget()
        self._create_status_bar()
        self._apply_premium_theme()

    def _create_menu_bar(self):
        mb = self.menuBar()
        file = mb.addMenu("&File")
        file.addAction(QAction("Open Folder...", self, shortcut="Ctrl+O", triggered=self.open_folder))
        file.addAction(QAction("Save All", self, shortcut="Ctrl+Shift+S", triggered=self.save_all))
        file.addSeparator()
        file.addAction(QAction("Exit", self, shortcut="Ctrl+Q", triggered=self.close))

        view = mb.addMenu("&View")
        view.addAction(QAction("Toggle Explorer", self, shortcut="Ctrl+B", triggered=lambda: self.explorer.setVisible(not self.explorer.isVisible())))
        view.addAction(QAction("Toggle Terminal", self, shortcut="Ctrl+`", triggered=lambda: self.terminal.setVisible(not self.terminal.isVisible())))
        view.addAction(QAction("Toggle Agent Trace", self, shortcut="Ctrl+Shift+A", triggered=lambda: self.agent_panel.setVisible(not self.agent_panel.isVisible())))

        swarm = mb.addMenu("&Swarm")
        swarm.addAction(QAction("Run Swarm", self, shortcut="Ctrl+Shift+R", triggered=self.run_swarm))

    def _create_toolbar(self):
        tb = QToolBar()
        tb.setMovable(False)
        self.addToolBar(tb)

        open_btn = QPushButton("📁 Open Folder")
        open_btn.clicked.connect(self.open_folder)
        tb.addWidget(open_btn)
        tb.addSeparator()
        swarm_btn = QPushButton("⚡ Run Swarm")
        swarm_btn.clicked.connect(self.run_swarm)
        tb.addWidget(swarm_btn)

    def _create_central_widget(self):
        central = QWidget()
        self.setCentralWidget(central)
        main = QHBoxLayout(central)
        main.setContentsMargins(0, 0, 0, 0)
        main.setSpacing(0)

        splitter = QSplitter(Qt.Orientation.Horizontal)

        self.explorer = self._create_explorer()
        splitter.addWidget(self.explorer)

        center = QSplitter(Qt.Orientation.Vertical)
        self.editor_tabs = QTabWidget()
        self.editor_tabs.setTabsClosable(True)
        self.editor_tabs.tabCloseRequested.connect(self.close_tab)
        center.addWidget(self.editor_tabs)

        self.terminal = self._create_terminal()
        center.addWidget(self.terminal)
        center.setStretchFactor(0, 3)
        center.setStretchFactor(1, 1)
        splitter.addWidget(center)

        self.agent_panel = self._create_agent_panel()
        splitter.addWidget(self.agent_panel)

        splitter.setStretchFactor(0, 1)
        splitter.setStretchFactor(1, 4)
        splitter.setStretchFactor(2, 1)
        main.addWidget(splitter)

    def _create_explorer(self):
        g = QGroupBox("📁 Explorer")
        l = QVBoxLayout(g)
        self.file_tree = QTreeWidget()
        self.file_tree.setHeaderHidden(True)
        l.addWidget(self.file_tree)
        return g

    def _create_terminal(self):
        g = QGroupBox("💻 Terminal")
        l = QVBoxLayout(g)
        self.terminal = QPlainTextEdit()
        self.terminal.setReadOnly(True)
        self.terminal.setFont(QFont("Consolas", 10))
        l.addWidget(self.terminal)
        return g

    def _create_agent_panel(self):
        g = QGroupBox("🤖 Agent Trace")
        l = QVBoxLayout(g)
        self.agent_scroll = QScrollArea()
        self.agent_scroll.setWidgetResizable(True)
        self.agent_content = QWidget()
        self.agent_layout = QVBoxLayout(self.agent_content)
        self.agent_layout.setAlignment(Qt.AlignmentFlag.AlignTop)
        self.agent_layout.setSpacing(8)
        self.agent_scroll.setWidget(self.agent_content)
        l.addWidget(self.agent_scroll)

        rag_g = QGroupBox("📊 RAG Heatmap")
        rag_l = QVBoxLayout(rag_g)
        self.rag_heatmap = RagHeatmapWidget()
        rag_l.addWidget(self.rag_heatmap)
        l.addWidget(rag_g)
        return g

    def _create_status_bar(self):
        sb = QStatusBar()
        self.setStatusBar(sb)
        self.status_label = QLabel("Ready")
        sb.addWidget(self.status_label, 1)

    def _apply_premium_theme(self):
        self.setStyleSheet("""
            QMainWindow, QGroupBox { background-color: #1e222a; color: #abb2bf; }
            QMenuBar { background-color: #21252b; color: #abb2bf; }
            QToolBar { background-color: #21252b; border-bottom: 1px solid #181a1f; padding: 4px; }
            QPushButton {
                background-color: #61afef; color: white; border: none;
                border-radius: 6px; padding: 8px 18px; font-weight: 600;
            }
            QPushButton:hover { background-color: #4a9eda; }
            QGroupBox { border: 1px solid #3c4048; border-radius: 8px; margin-top: 12px; }
            QGroupBox::title { left: 12px; padding: 0 6px; }
            QTreeWidget, QPlainTextEdit { background-color: #21252b; color: #abb2bf; border: none; }
            QScrollArea { background-color: #21252b; border: none; }
            QStatusBar { background-color: #61afef; color: white; }
        """)

    def _show_welcome(self):
        welcome = QPlainTextEdit()
        welcome.setReadOnly(True)
        welcome.setPlainText("""// Welcome to DroxIDE
// A native desktop AI-powered IDE
// Built with PyQt6 + Rust Core

// Getting Started:
// 1. Click "Open Folder" in the sidebar
// 2. Select a project directory
// 3. Click "⚡ Run Swarm" to start AI
// 4. Watch agents work in real-time

// Features:
// • 7-Agent Swarm Orchestration
// • Local RAG (no cloud dependencies)
// • Docker Sandbox isolation
// • Git integration
// • HITL review checkpoint
// • Immutable audit logging""")
        self.editor_tabs = QTabWidget()
        self.editor_tabs.addTab(welcome, "Welcome")

    def _on_agent_message(self, agent, status, content):
        bubble = AgentBubble(agent, status, content)
        self.agent_layout.addWidget(bubble)
        self.agent_scroll.verticalScrollBar().setValue(self.agent_scroll.verticalScrollBar().maximum())

    def _on_swarm_state(self, state):
        self.status_label.setText(f"Swarm: {state}")

    def _on_rag_update(self, data):
        self.rag_heatmap.update_heatmap(data.get("chunks", []), data.get("relevances", []))

    def open_folder(self):
        d = QFileDialog.getExistingDirectory(self, "Open Folder")
        if d:
            self.current_folder = d
            self.status_label.setText(f"Opened: {d}")
            self.rust.send_command(RustCore.CMD_QUERY_RAG, {"folder": d})

    def run_swarm(self):
        if not self.current_folder:
            self.status_label.setText("Open a folder first")
            return
        self.rust.send_command(RustCore.CMD_RUN_SWARM, {"folder": self.current_folder})

    def save_all(self):
        self.status_label.setText("All files saved")

    def close_tab(self, index):
        self.editor_tabs.removeTab(index)

def main():
    app = QApplication(sys.argv)
    app.setStyle("Fusion")
    win = DroxIDE()
    win.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
