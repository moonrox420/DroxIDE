// src/dialogs/runswarmdialog.cpp
#include "runswarmdialog.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QGroupBox>

RunSwarmDialog::RunSwarmDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle(QStringLiteral("Run Swarm"));
    setMinimumWidth(600);

    QVBoxLayout *layout = new QVBoxLayout(this);

    // Prompt section
    layout->addWidget(new QLabel(QStringLiteral("Prompt:")));
    mPromptArea = new QTextEdit();
    mPromptArea->setPlaceholderText(QStringLiteral("Enter your task or refactoring request..."));
    mPromptArea->setMaximumHeight(100);
    layout->addWidget(mPromptArea);

    // Context options
    QGroupBox *contextBox = new QGroupBox(QStringLiteral("Context"));
    QVBoxLayout *contextLayout = new QVBoxLayout(contextBox);

    mCurrentFileCheck = new QCheckBox(QStringLiteral("Current file"));
    mCurrentFileCheck->setChecked(true);
    contextLayout->addWidget(mCurrentFileCheck);

    mFolderCheck = new QCheckBox(QStringLiteral("Folder (src/)"));
    mFolderCheck->setChecked(true);
    contextLayout->addWidget(mFolderCheck);

    mGitHistoryCheck = new QCheckBox(QStringLiteral("Git history (last 10 commits)"));
    mGitHistoryCheck->setChecked(true);
    contextLayout->addWidget(mGitHistoryCheck);

    mDependenciesCheck = new QCheckBox(QStringLiteral("Dependencies (Cargo.toml)"));
    mDependenciesCheck->setChecked(true);
    contextLayout->addWidget(mDependenciesCheck);

    layout->addWidget(contextBox);

    // RAG Filters
    layout->addWidget(new QLabel(QStringLiteral("RAG Filters:")));
    QHBoxLayout *ragLayout = new QHBoxLayout();
    ragLayout->addWidget(new QPushButton(QStringLiteral("Tag Filter...")));
    ragLayout->addWidget(new QPushButton(QStringLiteral("Date Range...")));
    ragLayout->addStretch();
    layout->addLayout(ragLayout);

    // HITL options
    QGroupBox *hitlBox = new QGroupBox(QStringLiteral("HITL"));
    QVBoxLayout *hitlLayout = new QVBoxLayout(hitlBox);

    mShowTraceCheck = new QCheckBox(QStringLiteral("Show trace"));
    mShowTraceCheck->setChecked(true);
    hitlLayout->addWidget(mShowTraceCheck);

    mBlockOnReviewCheck = new QCheckBox(QStringLiteral("Block on review"));
    mBlockOnReviewCheck->setChecked(true);
    hitlLayout->addWidget(mBlockOnReviewCheck);

    mAutoApplyCheck = new QCheckBox(QStringLiteral("Auto-apply if confidence >90%"));
    hitlLayout->addWidget(mAutoApplyCheck);

    layout->addWidget(hitlBox);

    // Buttons
    QHBoxLayout *btnLayout = new QHBoxLayout();
    mCancelBtn = new QPushButton(QStringLiteral("Cancel"));
    mRunBtn = new QPushButton(QStringLiteral("Run"));
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
    if (mCurrentFileCheck->isChecked()) files << QStringLiteral("current");
    if (mFolderCheck->isChecked()) files << QStringLiteral("folder");
    if (mGitHistoryCheck->isChecked()) files << QStringLiteral("git");
    if (mDependenciesCheck->isChecked()) files << QStringLiteral("deps");
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
