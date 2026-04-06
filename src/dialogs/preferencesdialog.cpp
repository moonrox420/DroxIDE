// src/dialogs/preferencesdialog.cpp
#include "preferencesdialog.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QGroupBox>
#include <QPushButton>
#include <QSettings>

PreferencesDialog::PreferencesDialog(QWidget *parent)
    : QDialog(parent)
{
    setWindowTitle("Preferences");
    setMinimumWidth(500);
    
    QVBoxLayout *layout = new QVBoxLayout(this);
    
    // Editor settings
    QGroupBox *editorBox = new QGroupBox("Editor");
    QVBoxLayout *editorLayout = new QVBoxLayout(editorBox);
    
    editorLayout->addWidget(new QLabel("Theme:"));
    mThemeCombo = new QComboBox();
    mThemeCombo->addItems({"Light", "Dark", "High Contrast"});
    editorLayout->addWidget(mThemeCombo);
    
    editorLayout->addWidget(new QLabel("Font Size:"));
    mFontSizeSpinBox = new QSpinBox();
    mFontSizeSpinBox->setRange(8, 24);
    mFontSizeSpinBox->setValue(11);
    editorLayout->addWidget(mFontSizeSpinBox);
    
    mAutoSaveCheck = new QCheckBox("Auto-save on focus loss");
    mAutoSaveCheck->setChecked(true);
    editorLayout->addWidget(mAutoSaveCheck);
    
    mShowLineNumbersCheck = new QCheckBox("Show line numbers");
    mShowLineNumbersCheck->setChecked(true);
    editorLayout->addWidget(mShowLineNumbersCheck);
    
    layout->addWidget(editorBox);
    
    // RAG settings
    QGroupBox *ragBox = new QGroupBox("RAG Pipeline");
    QVBoxLayout *ragLayout = new QVBoxLayout(ragBox);
    
    ragLayout->addWidget(new QLabel("Embedding pool size:"));
    mRagPoolSizeSpinBox = new QSpinBox();
    mRagPoolSizeSpinBox->setRange(1, 16);
    mRagPoolSizeSpinBox->setValue(4);
    ragLayout->addWidget(mRagPoolSizeSpinBox);
    
    ragLayout->addWidget(new QLabel("Top-K documents:"));
    mDocTopKSpinBox = new QSpinBox();
    mDocTopKSpinBox->setRange(1, 50);
    mDocTopKSpinBox->setValue(5);
    ragLayout->addWidget(mDocTopKSpinBox);
    
    layout->addWidget(ragBox);
    
    layout->addStretch();
    
    // Buttons
    QHBoxLayout *btnLayout = new QHBoxLayout();
    QPushButton *cancelBtn = new QPushButton("Cancel");
    QPushButton *applyBtn = new QPushButton("Apply");
    QPushButton *okBtn = new QPushButton("OK");
    btnLayout->addStretch();
    btnLayout->addWidget(cancelBtn);
    btnLayout->addWidget(applyBtn);
    btnLayout->addWidget(okBtn);
    layout->addLayout(btnLayout);
    
    connect(cancelBtn, &QPushButton::clicked, this, &QDialog::reject);
    connect(applyBtn, &QPushButton::clicked, this, &PreferencesDialog::onApplyClicked);
    connect(okBtn, &QPushButton::clicked, this, &PreferencesDialog::onOkClicked);
    
    // Load settings
    QSettings settings("DroxIDE", "DroxIDE");
    mThemeCombo->setCurrentText(settings.value("theme", "Dark").toString());
    mFontSizeSpinBox->setValue(settings.value("fontSize", 11).toInt());
    mAutoSaveCheck->setChecked(settings.value("autoSave", true).toBool());
    mShowLineNumbersCheck->setChecked(settings.value("showLineNumbers", true).toBool());
}

void PreferencesDialog::onApplyClicked()
{
    QSettings settings("DroxIDE", "DroxIDE");
    settings.setValue("theme", mThemeCombo->currentText());
    settings.setValue("fontSize", mFontSizeSpinBox->value());
    settings.setValue("autoSave", mAutoSaveCheck->isChecked());
    settings.setValue("showLineNumbers", mShowLineNumbersCheck->isChecked());
    settings.setValue("ragPoolSize", mRagPoolSizeSpinBox->value());
    settings.setValue("docTopK", mDocTopKSpinBox->value());
}

void PreferencesDialog::onOkClicked()
{
    onApplyClicked();
    accept();
}
