// src/editor/editor.cpp
#include <QVBoxLayout>
#include <QFile>
#include <QTextStream>
#include <QFileInfo>
#include <QCloseEvent>
#include "editor.h"
#include "syntaxhighlighter.h"

Editor::Editor(QWidget *parent)
    : QWidget(parent)
    , m_textEdit(new QPlainTextEdit(this))
    , m_highlighter(nullptr)
    , m_modified(false)
    , m_saveTimer(new QTimer(this))
{
    setObjectName("Editor");

    auto *layout = new QVBoxLayout(this);
    layout->setContentsMargins(0, 0, 0, 0);
    layout->addWidget(m_textEdit);

    setupEditor();
    connectSignals();

    m_saveTimer->setSingleShot(true);
    m_saveTimer->setInterval(500);
    connect(m_saveTimer, &QTimer::timeout, this, &Editor::saveFile);
}

Editor::~Editor() = default;

void Editor::setupEditor()
{
    m_textEdit->setFont(QFont("Consolas", 11));
    m_textEdit->setTabStopDistance(4 * m_textEdit->fontMetrics().horizontalAdvance(' '));
    m_textEdit->setLineWrapMode(QPlainTextEdit::NoWrap);
    m_textEdit->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOn);

    m_highlighter = new SyntaxHighlighter(m_textEdit->document(), "cpp");
    setTheme(true);
}

void Editor::connectSignals()
{
    connect(m_textEdit->document(), &QTextDocument::modificationChanged,
            this, [this](bool modified) {
                m_modified = modified;
                emit modificationChanged(modified);
                updateTitle();
            });

    connect(m_textEdit, &QPlainTextEdit::textChanged, this, [this]() {
        if (!m_saveTimer->isActive()) m_saveTimer->start();
    });
}

void Editor::openFile(const QString &filePath)
{
    QFile file(filePath);
    if (file.open(QIODevice::ReadOnly | QIODevice::Text)) {
        m_textEdit->setPlainText(file.readAll());
        m_currentFilePath = filePath;
        m_textEdit->document()->setModified(false);
        emit fileChanged(filePath);
        updateTitle();
    }
}

void Editor::saveFile()
{
    if (m_currentFilePath.isEmpty()) return;

    QFile file(m_currentFilePath);
    if (file.open(QIODevice::WriteOnly | QIODevice::Text)) {
        QTextStream stream(&file);
        stream << m_textEdit->toPlainText();
        m_textEdit->document()->setModified(false);
    }
}

void Editor::saveFileAs(const QString &filePath)
{
    m_currentFilePath = filePath;
    saveFile();
}

void Editor::setFont(const QFont &font)
{
    m_textEdit->setFont(font);
    if (m_highlighter) m_highlighter->rehighlight();
}

void Editor::setTheme(bool dark)
{
    QPalette p = m_textEdit->palette();
    if (dark) {
        p.setColor(QPalette::Base, QColor(30, 30, 30));
        p.setColor(QPalette::Text, QColor(220, 220, 220));
    } else {
        p.setColor(QPalette::Base, Qt::white);
        p.setColor(QPalette::Text, Qt::black);
    }
    m_textEdit->setPalette(p);
}

void Editor::undo() { m_textEdit->undo(); }
void Editor::redo() { m_textEdit->redo(); }
void Editor::cut() { m_textEdit->cut(); }
void Editor::copy() { m_textEdit->copy(); }
void Editor::paste() { m_textEdit->paste(); }
void Editor::find() { /* TODO: implement find dialog if needed */ }

void Editor::closeEvent(QCloseEvent *event)
{
    if (m_textEdit->document()->isModified()) {
        saveFile();
    }
    QWidget::closeEvent(event);
}

void Editor::updateTitle()
{
    QString title = m_currentFilePath.isEmpty() ? "Untitled" : QFileInfo(m_currentFilePath).fileName();
    if (m_modified) title.prepend("*");
    setWindowTitle(title);
}