// src/panels/agenttracewidget.cpp
#include "agenttracewidget.h"
#include <QScrollBar>

AgentTraceWidget::AgentTraceWidget(QWidget *parent)
    : QWidget(parent)
{
    auto *mainLayout = new QVBoxLayout(this);
    mainLayout->setContentsMargins(0, 0, 0, 0);

    m_scroll = new QScrollArea(this);
    m_scroll->setWidgetResizable(true);
    mainLayout->addWidget(m_scroll);

    m_chatWidget = new QWidget();
    m_chatLayout = new QVBoxLayout(m_chatWidget);
    m_chatLayout->setAlignment(Qt::AlignTop);
    m_scroll->setWidget(m_chatWidget);
}

AgentTraceWidget::~AgentTraceWidget() = default;

void AgentTraceWidget::addMessage(const QString &json)
{
    QLabel *bubble = new QLabel(json);
    bubble->setWordWrap(true);
    bubble->setStyleSheet(QStringLiteral("QLabel { background-color: #2a2a2a; color: #ffffff; padding: 8px; border-radius: 8px; margin: 4px; }"));
    m_chatLayout->addWidget(bubble);
    m_scroll->verticalScrollBar()->setValue(m_scroll->verticalScrollBar()->maximum());
}