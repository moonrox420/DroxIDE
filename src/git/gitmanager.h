// src/git/gitmanager.h
#ifndef GITMANAGER_H
#define GITMANAGER_H

#include <QString>
#include <QStringList>

class GitManager {
public:
    GitManager();
    
    void commit(const QString &message);
    void push();
    void pull();
    void branch(const QString &name);
    void stash();
    QString blame(const QString &filePath, int lineNumber);

private:
    QString mRepoPath;
};

#endif // GITMANAGER_H
