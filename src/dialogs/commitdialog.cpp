// src/dialogs/commitdialog.cpp
#include "commitdialog.h"
#include "../git/gitmanager.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>

CommitDialog::CommitDialog(QWidget *parent, GitManager *gitManager)
    : QDialog(parent), mGitManager(gitManager)
{
    setWindowTitle(QStringLiteral("Git Commit"));
    setMinimumWidth(500);
    setMinimumHeight(400);

    QVBoxLayout *layout = new QVBoxLayout(this);

    layout->addWidget(new QLabel(QStringLiteral("Files to commit:")));

    mFilesList = new QListWidget();
    // TODO: Load files from git status
    mFilesList->addItem(QStringLiteral("main.rs"));
    mFilesList->addItem(QStringLiteral("Cargo.toml"));
    layout->addWidget(mFilesList);

    layout->addWidget(new QLabel(QStringLiteral("Commit Message:")));

    mMessageEdit = new QTextEdit();
    mMessageEdit->setPlaceholderText(QStringLiteral("Describe your changes..."));
    layout->addWidget(mMessageEdit);

    QHBoxLayout *btnLayout = new QHBoxLayout();
    QPushButton *cancelBtn = new QPushButton(QStringLiteral("Cancel"));
    mCommitBtn = new QPushButton(QStringLiteral("Commit"));
    btnLayout->addStretch();
    btnLayout->addWidget(cancelBtn);
    btnLayout->addWidget(mCommitBtn);
    layout->addLayout(btnLayout);

    connect(cancelBtn, &QPushButton::clicked, this, &QDialog::reject);
    connect(mCommitBtn, &QPushButton::clicked, this, &CommitDialog::onCommitClicked);
}

QString CommitDialog::getCommitMessage() const
{
    return mMessageEdit->toPlainText();
}

void CommitDialog::onCommitClicked()
{
    if (mMessageEdit->toPlainText().isEmpty()) {
        return;
    }
    
    if (mGitManager) {
        mGitManager->commit(mMessageEdit->toPlainText());
    }
    
    accept();
}
