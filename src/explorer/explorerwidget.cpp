// src/explorer/explorerwidget.cpp
#include "explorerwidget.h"
#include <QVBoxLayout>
#include <QDir>
#include <QFileInfo>
#include <QStandardPaths>

ExplorerWidget::ExplorerWidget(QWidget *parent)
    : QWidget(parent)
{
    QVBoxLayout *layout = new QVBoxLayout(this);
    
    mTreeWidget = new QTreeWidget();
    mTreeWidget->setHeaderLabel("Files");
    mTreeWidget->setContextMenuPolicy(Qt::CustomContextMenu);
    
    mWatcher = new QFileSystemWatcher(this);
    
    connect(mTreeWidget, &QTreeWidget::itemDoubleClicked, this, &ExplorerWidget::onItemDoubleClicked);
    connect(mWatcher, &QFileSystemWatcher::directoryChanged, this, &ExplorerWidget::onFolderChanged);
    connect(mWatcher, &QFileSystemWatcher::fileChanged, this, &ExplorerWidget::onFileChanged);
    
    layout->addWidget(mTreeWidget);
    layout->setContentsMargins(0, 0, 0, 0);
}

void ExplorerWidget::loadFolder(const QString &path)
{
    mRootPath = path;
    mTreeWidget->clear();
    
    QTreeWidgetItem *rootItem = new QTreeWidgetItem();
    rootItem->setText(0, QFileInfo(path).baseName());
    rootItem->setData(0, Qt::UserRole, path);
    mTreeWidget->addTopLevelItem(rootItem);
    
    populateTree(path, rootItem);
    mWatcher->addPath(path);
}

void ExplorerWidget::populateTree(const QString &path, QTreeWidgetItem *parent)
{
    QDir dir(path);
    dir.setFilter(QDir::NoDotAndDotDot | QDir::AllEntries);
    
    // Ignore common directories
    QStringList ignore = {".git", "node_modules", "__pycache__", "target", ".venv", ".idea"};
    
    for (const QFileInfo &info : dir.entryInfoList()) {
        if (ignore.contains(info.fileName())) continue;
        
        QTreeWidgetItem *item = new QTreeWidgetItem(parent);
        item->setText(0, info.fileName());
        item->setData(0, Qt::UserRole, info.filePath());
        
        if (info.isDir()) {
            item->setIcon(0, style()->standardIcon(QStyle::SP_DirIcon));
            populateTree(info.filePath(), item);
        } else {
            item->setIcon(0, style()->standardIcon(QStyle::SP_FileIcon));
        }
    }
}

void ExplorerWidget::onItemDoubleClicked(QTreeWidgetItem *item, int column)
{
    QString path = item->data(column, Qt::UserRole).toString();
    emit fileDoubleClicked(path);
}

void ExplorerWidget::onFolderChanged(const QString &path)
{
    // Refresh tree
}

void ExplorerWidget::onFileChanged(const QString &path)
{
    // Mark as modified
}
