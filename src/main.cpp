// src/main.cpp
#include <QApplication>
#include "mainwindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);

    app.setApplicationName(QStringLiteral("DroxIDE"));
    app.setApplicationVersion(QStringLiteral("1.0.0"));
    app.setApplicationDisplayName(QStringLiteral("DroxIDE - Native Desktop AI IDE"));

    MainWindow window;
    window.show();

    return app.exec();
}
