// src/terminal/terminalwidget.cpp
#include "terminalwidget.h"
#include <QDir>

TerminalWidget::TerminalWidget(QWidget *parent)
    : QWidget(parent)
    , m_process(new QProcess(this))
    , m_cursorTimer(new QTimer(this))
    , m_cursorVisible(true)
{
    auto *layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->setSpacing(0);

    m_terminal = new QPlainTextEdit(this);
    m_terminal->setReadOnly(false);
    m_terminal->setUndoRedoEnabled(false);
    m_terminal->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOn);
    m_terminal->setHorizontalScrollBarPolicy(Qt::ScrollBarAsNeeded);
    m_terminal->setFont(QFont("Consolas", 10));
    m_terminal->setStyleSheet("QPlainTextEdit { background-color: #1e1e1e; color: #d4d4d4; border: none; }");

    layout->addWidget(m_terminal);

    m_process->setProcessChannelMode(QProcess::MergedChannels);

    connect(m_process, &QProcess::readyReadStandardOutput, this, &TerminalWidget::onProcessReadyReadStandardOutput);
    connect(m_process, &QProcess::readyReadStandardError, this, &TerminalWidget::onProcessReadyReadStandardError);
    connect(m_process, &QProcess::finished, this, &TerminalWidget::onProcessFinished);

    m_cursorTimer->setInterval(500);
    connect(m_cursorTimer, &QTimer::timeout, this, [this]() {
        m_cursorVisible = !m_cursorVisible;
        m_terminal->viewport()->update();
    });
}

TerminalWidget::~TerminalWidget()
{
    if (m_process->state() != QProcess::NotRunning) {
        m_process->kill();
        m_process->waitForFinished(1000);
    }
}

void TerminalWidget::startShell(const QString &shell)
{
    if (m_process->state() != QProcess::NotRunning) return;

    QString program = shell.isEmpty() ? "cmd.exe" : shell;

    m_process->setProgram(program);
    m_process->setWorkingDirectory(m_workingDirectory.isEmpty() ? QDir::homePath() : m_workingDirectory);

    m_process->start();
    m_cursorTimer->start();
}

void TerminalWidget::sendInput(const QString &input)
{
    if (m_process->state() == QProcess::Running) {
        writeToProcess(input);
    }
}

void TerminalWidget::setWorkingDirectory(const QString &path)
{
    m_workingDirectory = path;
}

void TerminalWidget::setFont(const QFont &font)
{
    m_terminal->setFont(font);
}

void TerminalWidget::clear()
{
    m_terminal->clear();
}

void TerminalWidget::onProcessReadyReadStandardOutput()
{
    appendOutput(m_process->readAllStandardOutput());
}

void TerminalWidget::onProcessReadyReadStandardError()
{
    appendOutput(m_process->readAllStandardError(), true);
}

void TerminalWidget::onProcessFinished(int exitCode, QProcess::ExitStatus exitStatus)
{
    m_cursorTimer->stop();
    emit shellExited(exitCode);
}

void TerminalWidget::appendOutput(const QString &text, bool isError)
{
    if (text.isEmpty()) return;

    QTextCursor cursor = m_terminal->textCursor();
    cursor.movePosition(QTextCursor::End);

    if (isError) {
        cursor.insertHtml(QString("<span style=\"color:#ff5555;\">%1</span>").arg(text.toHtmlEscaped()));
    } else {
        cursor.insertText(text);
    }

    m_terminal->setTextCursor(cursor);
    m_terminal->ensureCursorVisible();
}

void TerminalWidget::writeToProcess(const QString &data)
{
    m_process->write(data.toUtf8());
}

void TerminalWidget::keyPressEvent(QKeyEvent *event)
{
    if (m_process->state() == QProcess::Running) {
        QString input = event->text();
        if (!input.isEmpty()) {
            writeToProcess(input);
            appendOutput(input);
        }
    }
    QWidget::keyPressEvent(event);
}

void TerminalWidget::resizeEvent(QResizeEvent *event)
{
    QWidget::resizeEvent(event);
}