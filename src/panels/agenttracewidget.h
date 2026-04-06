// src/panels/agenttracewidget.h
#pragma once

#include <QWidget>
#include <QVBoxLayout>
#include <QScrollArea>
#include <QLabel>
#include <QPushButton>

class AgentTraceWidget : public QWidget
{
    Q_OBJECT

public:
    explicit AgentTraceWidget(QWidget *parent = nullptr);
    ~AgentTraceWidget();

    void addMessage(const QString &json);

private:
    QScrollArea *m_scroll;
    QWidget *m_chatWidget;
    QVBoxLayout *m_chatLayout;
};