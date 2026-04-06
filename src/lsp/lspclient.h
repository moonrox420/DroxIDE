// src/lsp/lspclient.h
#ifndef LSPCLIENT_H
#define LSPCLIENT_H

#include <QString>
#include <QStringList>

class LspClient {
public:
    LspClient();
    
    void start();
    void stop();
    
    QStringList checkFile(const QString &filePath);
    QStringList getCompletions(const QString &filePath, int line, int col);
    QString getHover(const QString &filePath, int line, int col);

private:
    bool mStarted = false;
};

#endif // LSPCLIENT_H
