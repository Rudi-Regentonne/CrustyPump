import type { TimeRangePreset } from '../types';

interface TimeControlsProps {
  timeRangePreset: TimeRangePreset;
  customStart: string;
  customEnd: string;
  granularity: string;
  onTimeRangePresetChange: (preset: TimeRangePreset) => void;
  onCustomStartChange: (val: string) => void;
  onCustomEndChange: (val: string) => void;
  onGranularityChange: (val: string) => void;
  onExportPdf: () => void;
}

export function TimeControls({
  timeRangePreset,
  customStart,
  customEnd,
  granularity,
  onTimeRangePresetChange,
  onCustomStartChange,
  onCustomEndChange,
  onGranularityChange,
  onExportPdf,
}: TimeControlsProps) {
  return (
    <div id="time-controls">
      <div className="control-group">
        <label htmlFor="time-range-preset">Time Range:</label>
        <select
          id="time-range-preset"
          value={timeRangePreset}
          onChange={(e) => onTimeRangePresetChange(e.target.value as TimeRangePreset)}
        >
          <option value="1m">Last 1 Minute</option>
          <option value="2m">Last 2 Minutes</option>
          <option value="15m">Last 15 Minutes</option>
          <option value="1h">Last 1 Hour</option>
          <option value="24h">Last 24 Hours</option>
          <option value="7d">Last 7 Days</option>
          <option value="custom">Custom</option>
        </select>
      </div>

      {timeRangePreset === 'custom' && (
        <div id="custom-time-container" className="control-group">
          <div className="custom-time-field">
            <label htmlFor="datetime-start">Start:</label>
            <input
              type="datetime-local"
              id="datetime-start"
              step="1"
              value={customStart}
              onChange={(e) => onCustomStartChange(e.target.value)}
            />
          </div>
          <div className="custom-time-field">
            <label htmlFor="datetime-end">End:</label>
            <input
              type="datetime-local"
              id="datetime-end"
              step="1"
              value={customEnd}
              onChange={(e) => onCustomEndChange(e.target.value)}
            />
          </div>
        </div>
      )}

      <div className="control-group">
        <label htmlFor="granularity">Granularity:</label>
        <select
          id="granularity"
          value={granularity}
          onChange={(e) => onGranularityChange(e.target.value)}
        >
          <option value="1 minute">1 Minute</option>
          <option value="15 minutes">15 Minutes</option>
          <option value="1 hour">1 Hour</option>
          <option value="1 day">1 Day</option>
        </select>
      </div>

      <div className="control-actions">
        <button id="export-btn" className="btn danger export" type="button" onClick={onExportPdf}>
          PDF Export
        </button>
      </div>
    </div>
  );
}
