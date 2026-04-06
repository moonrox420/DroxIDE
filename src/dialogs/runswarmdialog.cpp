// src/dialogs/runswarmdialog.cpp
#include "runswarmdialog.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QGroupBox>

RunSwarmDialog::RunSwarmDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle("Run Swarm");
    setMinimumWidth(600);
    
    QVBoxLayout *layout = new QVBoxLayout(this);
    
    // Prompt section
    layout->addWidget(new QLabel("Prompt:"));
    mPromptArea = new QTextEdit();
    mPromptArea->setPlaceholderText("Enter your task or refactoring request...");
    mPromptArea->setMaximumHeight(100);
    layout->addWidget(mPromptArea);
    
    // Context options
    QGroupBox *contextBox = new QGroupBox("Context");
    QVBoxLayout *contextLayout = new QVBoxLayout(contextBox);
    
    mCurrentFileCheck = new QCheckBox("Current file");
    mCurrentFileCheck->setChecked(true);
    contextLayout->addWidget(mCurrentFileCheck);
    
    mFolderCheck = new QCheckBox("Folder (src/)");
    mFolderCheck->setChecked(true);
    contextLayout->addWidget(mFolderCheck);
    
    mGitHistoryCheck = new QCheckBox("Git history (last 10 commits)");
    mGitHistoryCheck->setChecked(true);
    contextLayout->addWidget(mGitHistoryCheck);
    
    mDependenciesCheck = new QCheckBox("Dependencies (Cargo.toml)");
    mDependenciesCheck->setChecked(true);
    contextLayout->addWidget(mDependenciesCheck);
    
    layout->addWidget(contextBox);
    
    // RAG Filters
    layout->addWidget(new QLabel("RAG Filters:"));
    QHBoxLayout *ragLayout = new QHBoxLayout();
    ragLayout->addWidget(new QPushButton("Tag Filter..."));
    ragLayout->addWidget(new QPushButton("Date Range..."));
    ragLayout->addStretch();
    layout->addLayout(ragLayout);
    
    // HITL options
    QGroupBox *hitlBox = new QGroupBox("HITL");
    QVBoxLayout *hitlLayout = new QVBoxLayout(hitlBox);
    
    mShowTraceCheck = new QCheckBox("Show trace");
    mShowTraceCheck->setChecked(true);
    hitlLayout->addWidget(mShowTraceCheck);
    
    mBlockOnReviewCheck = new QCheckBox("Block on review");
    mBlockOnReviewCheck->setChecked(true);
    hitlLayout->addWidget(mBlockOnReviewCheck);
    
    mAutoApplyCheck = new QCheckBox("Auto-apply if confidence >90%");
    hitlLayout->addWidget(mAutoApplyCheck);
    
    layout->addWidget(hitlBox);
    
    // Buttons
    QHBoxLayout *btnLayout = new QHBoxLayout();
    mCancelBtn = new QPushButton("Cancel");
    mRunBtn = new QPushButton("Run");
    btnLayout->addStretch();
    btnLayout->addWidget(mCancelBtn);
    btnLayout->addWidget(mRunBtn);
    layout->addLayout(btnLayout);
    
    connect(mCancelBtn, &QPushButton::clicked, this, &QDialog::reject);
    connect(mRunBtn, &QPushButton::clicked, this, &RunSwarmDialog::onRunClicked);
}

QString RunSwarmDialog::getPrompt() const
{
    return mPromptArea->toPlainText();
}

QStringList RunSwarmDialog::getContextFiles() const
{
    QStringList files;
    if (mCurrentFileCheck->isChecked()) files << "current";
    if (mFolderCheck->isChecked()) files << "folder";
    if (mGitHistoryCheck->isChecked()) files << "git";
    if (mDependenciesCheck->isChecked()) files << "deps";
    return files;
}

bool RunSwarmDialog::blockOnReview() const
{
    return mBlockOnReviewCheck->isChecked();
}

bool RunSwarmDialog::autoApplyIfHighConfidence() const
{
    return mAutoApplyCheck->isChecked();
}

void RunSwarmDialog::onRunClicked()
{
    if (mPromptArea->toPlainText().isEmpty()) {
        return;
    }
    accept();
}
