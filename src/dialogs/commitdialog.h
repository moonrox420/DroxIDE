// src/dialogs/commitdialog.h
#ifndef COMMITDIALOG_H
#define COMMITDIALOG_H

#include <QDialog>
#include <QTextEdit>
#include <QListWidget>
#include <QPushButton>

class GitManager;

class CommitDialog : public QDialog {
    Q_OBJECT

public:
    CommitDialog(QWidget *parent, GitManager *gitManager);
    
    QString getCommitMessage() const;

private Q_SLOTS:
    void onCommitClicked();

private:
    QListWidget *mFilesList = nullptr;
    QTextEdit *mMessageEdit = nullptr;
    QPushButton *mCommitBtn = nullptr;
    GitManager *mGitManager = nullptr;
};

#endif // COMMITDIALOG_H
