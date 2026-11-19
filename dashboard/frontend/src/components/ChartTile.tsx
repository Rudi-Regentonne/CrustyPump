import { useEffect, useRef } from 'react';
import {
  Chart,
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  TimeScale,
  Filler,
  Tooltip,
} from 'chart.js';
import 'chartjs-adapter-date-fns';
import type { DataPoint, TileConfig, TimeRangePreset } from '../types';
import { getStartEndDates } from '../utils/time';
import type { PdfChartEntry } from '../utils/pdfExport';

Chart.register(
  LineController,
  LineElement,
  PointElement,
  LinearScale,
  TimeScale,
  Filler,
  Tooltip,
);

interface ChartTileProps {
  tile: TileConfig;
  timeRangePreset: TimeRangePreset;
  customStart: string;
  customEnd: string;
  granularity: string;
  refreshTick: number;
  onRemove: () => void;
  registerPdfEntry: (key: string, entry: PdfChartEntry | null) => void;
}

export function ChartTile({
  tile,
  timeRangePreset,
  customStart,
  customEnd,
  granularity,
  refreshTick,
  onRemove,
  registerPdfEntry,
}: ChartTileProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const chartRef = useRef<Chart | null>(null);

  const tileKey = `${tile.measurement}:${tile.channel}`;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const chart = new Chart(ctx, {
      type: 'line',
      data: { datasets: [] },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: {
          mode: 'index',
          intersect: false,
        },
        scales: {
          x: {
            type: 'time',
            time: { tooltipFormat: 'PPpp' },
            title: { display: false },
          },
          y: {
            title: { display: true, text: 'Value' },
          },
        },
        plugins: {
          legend: { display: false },
        },
      },
    });

    chartRef.current = chart;
    registerPdfEntry(tileKey, {
      title: tile.channel,
      chart,
      canvas,
      container: containerRef.current,
    });

    return () => {
      registerPdfEntry(tileKey, null);
      chart.destroy();
      chartRef.current = null;
    };
  }, [tile.channel, tileKey, registerPdfEntry]);

  useEffect(() => {
    let cancelled = false;
    const dates = getStartEndDates(timeRangePreset, customStart, customEnd);
    if (!dates.start || !dates.end) return;

    const url = `/api/data?channel=${encodeURIComponent(tile.channel)}&measurement=${encodeURIComponent(tile.measurement)}&start=${encodeURIComponent(dates.start)}&end=${encodeURIComponent(dates.end)}&granularity=${encodeURIComponent(granularity)}`;

    fetch(url)
      .then((res) => res.json())
      .then((data: DataPoint[]) => {
        if (cancelled) return;
        const chart = chartRef.current;
        if (!chart) return;

        const chartData = data
          .filter((d) => d.val !== null)
          .map((d) => ({
            x: new Date(d.bucket).getTime(),
            y: d.val as number,
          }));

        chart.data.datasets = [
          {
            label: tile.channel,
            data: chartData,
            borderColor: '#1976D2',
            backgroundColor: 'rgba(25, 118, 210, 0.1)',
            borderWidth: 2,
            pointRadius: 0,
            pointHitRadius: 10,
            fill: true,
            tension: 0.1,
          },
        ];
        chart.update();
      })
      .catch((e) => {
        console.error('Failed to load chart data', e);
      });

    return () => {
      cancelled = true;
    };
  }, [tile.channel, tile.measurement, timeRangePreset, customStart, customEnd, granularity, refreshTick]);

  return (
    <div className="tile" data-tile-key={tileKey}>
      <div className="tile-header">
        <div className="tile-title">{tile.channel}</div>
        <div className="tile-controls">
          <button
            className="remove"
            type="button"
            aria-label={`Remove ${tile.channel}`}
            onClick={onRemove}
          >
            ×
          </button>
        </div>
      </div>
      <div className="chart-container" ref={containerRef}>
        <canvas ref={canvasRef} />
      </div>
    </div>
  );
}
