// src/panels/ragheatmapwidget.cpp
#include "ragheatmapwidget.h"

RagHeatmapWidget::RagHeatmapWidget(QWidget *parent)
    : QWidget(parent)
{
    setMinimumHeight(200);
}

RagHeatmapWidget::~RagHeatmapWidget() = default;

void RagHeatmapWidget::updateHeatmap(const QStringList &chunks, const QVector<float> &relevances)
{
    m_chunks = chunks;
    m_relevances = relevances;
    update();
}

void RagHeatmapWidget::paintEvent(QPaintEvent *event)
{
    QPainter p(this);
    p.fillRect(rect(), Qt::black);

    if (m_chunks.isEmpty()) return;

    int barHeight = height() / m_chunks.size();
    for (int i = 0; i < m_chunks.size(); ++i) {
        float relevance = i < m_relevances.size() ? m_relevances[i] : 0.0f;
        int barWidth = static_cast<int>(width() * relevance);

        QColor color = QColor::fromHsvF(0.3f * relevance, 1.0f, 0.8f);
        p.fillRect(0, i * barHeight, barWidth, barHeight - 1, color);

        p.setPen(Qt::white);
        p.drawText(5, i * barHeight + barHeight / 2 + 5, m_chunks[i]);
    }
}