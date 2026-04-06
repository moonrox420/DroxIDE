// src/panels/ragheatmapwidget.h
#pragma once

#include <QWidget>
#include <QPainter>
#include <QScrollArea>

class RagHeatmapWidget : public QWidget
{
    Q_OBJECT

public:
    explicit RagHeatmapWidget(QWidget *parent = nullptr);
    ~RagHeatmapWidget();

    void updateHeatmap(const QStringList &chunks, const QVector<float> &relevances);

protected:
    void paintEvent(QPaintEvent *event) override;

private:
    QStringList m_chunks;
    QVector<float> m_relevances;
};