// src/dialogs/preferencesdialog.h
#ifndef PREFERENCESDIALOG_H
#define PREFERENCESDIALOG_H

#include <QDialog>
#include <QSpinBox>
#include <QCheckBox>
#include <QComboBox>

class PreferencesDialog : public QDialog {
    Q_OBJECT

public:
    PreferencesDialog(QWidget *parent = nullptr);

private Q_SLOTS:
    void onApplyClicked();
    void onOkClicked();

private:
    QComboBox *mThemeCombo = nullptr;
    QSpinBox *mFontSizeSpinBox = nullptr;
    QCheckBox *mAutoSaveCheck = nullptr;
    QCheckBox *mShowLineNumbersCheck = nullptr;
    QSpinBox *mRagPoolSizeSpinBox = nullptr;
    QSpinBox *mDocTopKSpinBox = nullptr;
};

#endif // PREFERENCESDIALOG_H
