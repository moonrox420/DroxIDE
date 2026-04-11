// src/dialogs/runswarmdialog.h
#ifndef RUNSWARMDIALOG_H
#define RUNSWARMDIALOG_H

#include <QDialog>
#include <QLineEdit>
#include <QTextEdit>
#include <QCheckBox>
#include <QPushButton>
#include <QComboBox>

class RunSwarmDialog : public QDialog {
    Q_OBJECT

public:
    RunSwarmDialog(QWidget *parent = nullptr);
    
    QString getPrompt() const;
    QStringList getContextFiles() const;
    bool blockOnReview() const;
    bool autoApplyIfHighConfidence() const;

private Q_SLOTS:
    void onRunClicked();

private:
    QLineEdit *mPromptEdit = nullptr;
    QTextEdit *mPromptArea = nullptr;
    QCheckBox *mCurrentFileCheck = nullptr;
    QCheckBox *mFolderCheck = nullptr;
    QCheckBox *mGitHistoryCheck = nullptr;
    QCheckBox *mDependenciesCheck = nullptr;
    QCheckBox *mShowTraceCheck = nullptr;
    QCheckBox *mBlockOnReviewCheck = nullptr;
    QCheckBox *mAutoApplyCheck = nullptr;
    QPushButton *mRunBtn = nullptr;
    QPushButton *mCancelBtn = nullptr;
};

#endif // RUNSWARMDIALOG_H
