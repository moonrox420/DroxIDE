// src/main.cpp
#include <QApplication>
#include "mainwindow.h"

int main(int argc, char *argv[])
{
    QApplication app(argc, argv);
    
    app.setApplicationName("DroxIDE");
    app.setApplicationVersion("1.0.0");
    app.setApplicationDisplayName("DroxIDE - Native Desktop AI IDE");
    
    MainWindow window;
    window.show();
    
    return app.exec();
}
