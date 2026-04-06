// src/terminal/terminalwidget.h
#pragma once

#include <QWidget>
#include <QPlainTextEdit>
#include <QProcess>
#include <QTextCursor>
#include <QTimer>
#include <QVBoxLayout>
#include <QKeyEvent>

class TerminalWidget : public QWidget
{
    Q_OBJECT

public:
    explicit TerminalWidget(QWidget *parent = nullptr);
    ~TerminalWidget();

    void startShell(const QString &shell = QString());
    void sendInput(const QString &input);
    void setWorkingDirectory(const QString &path);
    void setFont(const QFont &font);
    void clear();

signals:
    void outputReceived(const QString &text);
    void shellExited(int exitCode);

public slots:
    void onProcessReadyReadStandardOutput();
    void onProcessReadyReadStandardError();
    void onProcessFinished(int exitCode, QProcess::ExitStatus exitStatus);

protected:
    void keyPressEvent(QKeyEvent *event) override;
    void resizeEvent(QResizeEvent *event) override;

private:
    QPlainTextEdit *m_terminal;
    QProcess *m_process;
    QString m_workingDirectory;
    QTimer *m_cursorTimer;
    bool m_cursorVisible;

    void appendOutput(const QString &text, bool isError = false);
    void writeToProcess(const QString &data);
};