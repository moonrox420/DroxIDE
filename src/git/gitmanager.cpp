// src/git/gitmanager.cpp
#include "gitmanager.h"
#include <QProcess>
#include <QDir>

GitManager::GitManager()
    : mRepoPath(QDir::currentPath())
{
}

void GitManager::commit(const QString &message)
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("commit") << QStringLiteral("-m") << message);
    proc.waitForFinished();
}

void GitManager::push()
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("push"));
    proc.waitForFinished();
}

void GitManager::pull()
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("pull"));
    proc.waitForFinished();
}

void GitManager::branch(const QString &name)
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("checkout") << QStringLiteral("-b") << name);
    proc.waitForFinished();
}

void GitManager::stash()
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("stash"));
    proc.waitForFinished();
}

QString GitManager::blame(const QString &filePath, int lineNumber)
{
    QProcess proc;
    proc.setWorkingDirectory(mRepoPath);
    proc.start(QStringLiteral("git"), QStringList() << QStringLiteral("blame") << QStringLiteral("-L") << QStringLiteral("%1,%1").arg(lineNumber) << filePath);
    proc.waitForFinished();
    return QString::fromLocal8Bit(proc.readAllStandardOutput());
}
