// src/explorer/explorerwidget.h
#ifndef EXPLORERWIDGET_H
#define EXPLORERWIDGET_H

#include <QtWidgets>
#include <QtCore>

class ExplorerWidget : public QWidget {
    Q_OBJECT

public:
    explicit ExplorerWidget(QWidget *parent = nullptr);

    void loadFolder(const QString &path);

signals:
    void fileSelected(const QString &filePath);
    void fileDoubleClicked(const QString &filePath);

private Q_SLOTS:
    void onItemDoubleClicked(QTreeWidgetItem *item, int column);
    void onFolderChanged(const QString &path);
    void onFileChanged(const QString &path);

private:
    void populateTree(const QString &path, QTreeWidgetItem *parent = nullptr);

    QTreeWidget *mTreeWidget = nullptr;
    QFileSystemWatcher *mWatcher = nullptr;
    QString mRootPath;
};

#endif // EXPLORERWIDGET_H
