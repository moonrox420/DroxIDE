// src/lsp/lspclient.cpp
#include "lspclient.h"

LspClient::LspClient()
{
}

void LspClient::start()
{
    // TODO: Start LSP server (Pyright, rust-analyzer, tsserver)
    // TODO: JSON-RPC communication
    mStarted = true;
}

void LspClient::stop()
{
    mStarted = false;
}

QStringList LspClient::checkFile(const QString &filePath)
{
    // TODO: diagnostics request
    return QStringList();
}

QStringList LspClient::getCompletions(const QString &filePath, int line, int col)
{
    // TODO: completion request
    return QStringList();
}

QString LspClient::getHover(const QString &filePath, int line, int col)
{
    // TODO: hover request
    return QString();
}
