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
    setWindowTitle(QStringLiteral("Preferences"));
    setMinimumWidth(500);

    QVBoxLayout *layout = new QVBoxLayout(this);

    // Editor settings
    QGroupBox *editorBox = new QGroupBox(QStringLiteral("Editor"));
    QVBoxLayout *editorLayout = new QVBoxLayout(editorBox);

    editorLayout->addWidget(new QLabel(QStringLiteral("Theme:")));
    mThemeCombo = new QComboBox();
    mThemeCombo->addItems({QStringLiteral("Light"), QStringLiteral("Dark"), QStringLiteral("High Contrast")});
    editorLayout->addWidget(mThemeCombo);

    editorLayout->addWidget(new QLabel(QStringLiteral("Font Size:")));
    mFontSizeSpinBox = new QSpinBox();
    mFontSizeSpinBox->setRange(8, 24);
    mFontSizeSpinBox->setValue(11);
    editorLayout->addWidget(mFontSizeSpinBox);

    mAutoSaveCheck = new QCheckBox(QStringLiteral("Auto-save on focus loss"));
    mAutoSaveCheck->setChecked(true);
    editorLayout->addWidget(mAutoSaveCheck);

    mShowLineNumbersCheck = new QCheckBox(QStringLiteral("Show line numbers"));
    mShowLineNumbersCheck->setChecked(true);
    editorLayout->addWidget(mShowLineNumbersCheck);

    layout->addWidget(editorBox);

    // RAG settings
    QGroupBox *ragBox = new QGroupBox(QStringLiteral("RAG Pipeline"));
    QVBoxLayout *ragLayout = new QVBoxLayout(ragBox);

    ragLayout->addWidget(new QLabel(QStringLiteral("Embedding pool size:")));
    mRagPoolSizeSpinBox = new QSpinBox();
    mRagPoolSizeSpinBox->setRange(1, 16);
    mRagPoolSizeSpinBox->setValue(4);
    ragLayout->addWidget(mRagPoolSizeSpinBox);

    ragLayout->addWidget(new QLabel(QStringLiteral("Top-K documents:")));
    mDocTopKSpinBox = new QSpinBox();
    mDocTopKSpinBox->setRange(1, 50);
    mDocTopKSpinBox->setValue(5);
    ragLayout->addWidget(mDocTopKSpinBox);

    layout->addWidget(ragBox);

    layout->addStretch();

    // Buttons
    QHBoxLayout *btnLayout = new QHBoxLayout();
    QPushButton *cancelBtn = new QPushButton(QStringLiteral("Cancel"));
    QPushButton *applyBtn = new QPushButton(QStringLiteral("Apply"));
    QPushButton *okBtn = new QPushButton(QStringLiteral("OK"));
    btnLayout->addStretch();
    btnLayout->addWidget(cancelBtn);
    btnLayout->addWidget(applyBtn);
    btnLayout->addWidget(okBtn);
    layout->addLayout(btnLayout);

    connect(cancelBtn, &QPushButton::clicked, this, &QDialog::reject);
    connect(applyBtn, &QPushButton::clicked, this, &PreferencesDialog::onApplyClicked);
    connect(okBtn, &QPushButton::clicked, this, &PreferencesDialog::onOkClicked);

    // Load settings
    QSettings settings(QStringLiteral("DroxIDE"), QStringLiteral("DroxIDE"));
    mThemeCombo->setCurrentText(settings.value(QStringLiteral("theme"), QStringLiteral("Dark")).toString());
    mFontSizeSpinBox->setValue(settings.value(QStringLiteral("fontSize"), 11).toInt());
    mAutoSaveCheck->setChecked(settings.value(QStringLiteral("autoSave"), true).toBool());
    mShowLineNumbersCheck->setChecked(settings.value(QStringLiteral("showLineNumbers"), true).toBool());
}

void PreferencesDialog::onApplyClicked()
{
    QSettings settings(QStringLiteral("DroxIDE"), QStringLiteral("DroxIDE"));
    settings.setValue(QStringLiteral("theme"), mThemeCombo->currentText());
    settings.setValue(QStringLiteral("fontSize"), mFontSizeSpinBox->value());
    settings.setValue(QStringLiteral("autoSave"), mAutoSaveCheck->isChecked());
    settings.setValue(QStringLiteral("showLineNumbers"), mShowLineNumbersCheck->isChecked());
    settings.setValue(QStringLiteral("ragPoolSize"), mRagPoolSizeSpinBox->value());
    settings.setValue(QStringLiteral("docTopK"), mDocTopKSpinBox->value());
}

void PreferencesDialog::onOkClicked()
{
    onApplyClicked();
    accept();
}
