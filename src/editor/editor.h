// src/editor/editor.h
#pragma once

#include <QtWidgets>
#include <QtCore>

class SyntaxHighlighter;

class Editor : public QWidget
{
    Q_OBJECT

public:
    explicit Editor(QWidget *parent = nullptr);
    ~Editor();

    void openFile(const QString &filePath);
    void saveFile();
    void saveFileAs(const QString &filePath);
    void setFont(const QFont &font);
    void setTheme(bool dark);

    QString currentFilePath() const { return m_currentFilePath; }
    QPlainTextEdit *textEdit() const { return m_textEdit; }

signals:
    void fileChanged(const QString &path);
    void modificationChanged(bool modified);

public Q_SLOTS:
    void undo();
    void redo();
    void cut();
    void copy();
    void paste();
    void find();

protected:
    void closeEvent(QCloseEvent *event) override;

private:
    QPlainTextEdit *m_textEdit;
    QSyntaxHighlighter *m_highlighter;
    QString m_currentFilePath;
    bool m_modified;
    QTimer *m_saveTimer;

    void setupEditor();
    void connectSignals();
    void documentModified();
    void updateTitle();
};
