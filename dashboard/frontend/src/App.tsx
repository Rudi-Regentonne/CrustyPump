import { useCallback, useEffect, useRef, useState } from 'react';
import Sortable from 'sortablejs';
import type { Channel, DashboardConfig, TileConfig, TimeRangePreset } from './types';
import { toLocalISOString } from './utils/time';
import { exportChartsToPdf, type PdfChartEntry } from './utils/pdfExport';
import { Header } from './components/Header';
import { TimeControls } from './components/TimeControls';
import { Sidebar } from './components/Sidebar';
import { ChartTile } from './components/ChartTile';

export function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [selectedChannel, setSelectedChannel] = useState('');
  const [tiles, setTiles] = useState<TileConfig[]>([]);
  const [timeRangePreset, setTimeRangePreset] = useState<TimeRangePreset>('1h');
  const [customStart, setCustomStart] = useState('');
  const [customEnd, setCustomEnd] = useState('');
  const [granularity, setGranularity] = useState('1 minute');
  const [refreshTick, setRefreshTick] = useState(0);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [statusMsg, setStatusMsg] = useState<{ text: string; isError: boolean } | null>(null);

  const tilesContainerRef = useRef<HTMLDivElement | null>(null);
  const pdfEntriesRef = useRef<Map<string, PdfChartEntry>>(new Map());
  const statusTimerRef = useRef<number | null>(null);

  const showStatus = useCallback((text: string, isError = false) => {
    setStatusMsg({ text, isError });
    if (statusTimerRef.current !== null) {
      window.clearTimeout(statusTimerRef.current);
    }
    statusTimerRef.current = window.setTimeout(() => {
      setStatusMsg(null);
    }, 3000);
  }, []);

  const loadChannels = useCallback(async () => {
    try {
      const res = await fetch('/api/channels');
      const data: Channel[] = await res.json();
      setChannels(data);
    } catch (e) {
      console.error('Failed to load channels', e);
    }
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      const res = await fetch('/api/config');
      const config: DashboardConfig = await res.json();

      if (config.time_range_preset) {
        setTimeRangePreset(config.time_range_preset);
      }
      if (config.time_range_preset === 'custom' && config.time_range) {
        setCustomStart(config.time_range.start);
        setCustomEnd(config.time_range.end);
      }
      if (config.granularity) {
        setGranularity(config.granularity);
      }
      const nextTiles = config.tiles ?? [];
      setTiles(nextTiles);
      if (nextTiles.length === 0 && window.innerWidth <= 768) {
        setSidebarOpen(true);
      }
    } catch (e) {
      console.error('Failed to load config', e);
    }
  }, []);

  useEffect(() => {
    void loadChannels();
    void loadConfig();
  }, [loadChannels, loadConfig]);

  useEffect(() => {
    const container = tilesContainerRef.current;
    if (!container) return;

    const sortable = new Sortable(container, {
      handle: '.tile-header',
      filter: '.tile-controls',
      preventOnFilter: false,
      delay: 150,
      delayOnTouchOnly: true,
      animation: 150,
      onEnd: (evt) => {
        const { oldIndex, newIndex } = evt;
        if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) {
          return;
        }
        setTiles((prev) => {
          const next = [...prev];
          const [moved] = next.splice(oldIndex, 1);
          if (moved) {
            next.splice(newIndex, 0, moved);
          }
          return next;
        });
      },
    });

    return () => {
      sortable.destroy();
    };
  }, []);

  const handleTimeRangePresetChange = (preset: TimeRangePreset) => {
    setTimeRangePreset(preset);
    if (preset === 'custom' && !customStart) {
      const now = new Date();
      const past = new Date(now.getTime() - 60 * 60 * 1000);
      setCustomEnd(toLocalISOString(now));
      setCustomStart(toLocalISOString(past));
    }
  };

  const handleAddTile = () => {
    if (!selectedChannel) return;
    const sep = selectedChannel.indexOf(':');
    if (sep === -1) return;
    const measurement = selectedChannel.slice(0, sep);
    const channel = selectedChannel.slice(sep + 1);

    setTiles((prev) => {
      if (prev.some((t) => t.channel === channel && t.measurement === measurement)) {
        return prev;
      }
      return [...prev, { channel, measurement }];
    });
  };

  const handleRemoveTile = (measurement: string, channel: string) => {
    setTiles((prev) => {
      const next = prev.filter((t) => !(t.channel === channel && t.measurement === measurement));
      if (next.length === 0 && window.innerWidth <= 768) {
        setSidebarOpen(true);
      }
      return next;
    });
  };

  const handleSaveConfig = async () => {
    const config: DashboardConfig = {
      time_range_preset: timeRangePreset,
      time_range: {
        start: customStart,
        end: customEnd,
      },
      granularity,
      tiles,
    };

    try {
      const res = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      if (res.ok) {
        showStatus('Layout saved');
      } else {
        showStatus('Failed to save', true);
      }
    } catch {
      showStatus('Error saving', true);
    }
  };

  const handleReloadConfig = async () => {
    setTiles([]);
    await loadConfig();
    setRefreshTick((t) => t + 1);
  };

  const registerPdfEntry = useCallback((key: string, entry: PdfChartEntry | null) => {
    if (entry) {
      pdfEntriesRef.current.set(key, entry);
    } else {
      pdfEntriesRef.current.delete(key);
    }
  }, []);

  const handleExportPdf = () => {
    if (tiles.length === 0) {
      showStatus('No charts to export', true);
      return;
    }

    const entries = tiles
      .map((t) => pdfEntriesRef.current.get(`${t.measurement}:${t.channel}`))
      .filter((entry): entry is PdfChartEntry => entry !== undefined);

    if (!exportChartsToPdf(entries)) {
      showStatus('No charts to export', true);
    }
  };

  return (
    <div id="app">
      <Header
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen((open) => !open)}
      />

      <TimeControls
        timeRangePreset={timeRangePreset}
        customStart={customStart}
        customEnd={customEnd}
        granularity={granularity}
        onTimeRangePresetChange={handleTimeRangePresetChange}
        onCustomStartChange={setCustomStart}
        onCustomEndChange={setCustomEnd}
        onGranularityChange={setGranularity}
        onExportPdf={handleExportPdf}
      />

      <div id="body-container">
        <Sidebar
          channels={channels}
          selectedChannel={selectedChannel}
          onSelectChannel={setSelectedChannel}
          onAddTile={handleAddTile}
          onSaveConfig={handleSaveConfig}
          onReloadConfig={handleReloadConfig}
          statusMsg={statusMsg}
          isOpen={sidebarOpen}
        />

        <main id="main-area">
          <div id="tiles-container" ref={tilesContainerRef}>
            {tiles.length === 0 ? (
              <p id="empty-state">Select channels from the sidebar to add graphs</p>
            ) : (
              tiles.map((t) => (
                <ChartTile
                  key={`${t.measurement}:${t.channel}`}
                  tile={t}
                  timeRangePreset={timeRangePreset}
                  customStart={customStart}
                  customEnd={customEnd}
                  granularity={granularity}
                  refreshTick={refreshTick}
                  onRemove={() => handleRemoveTile(t.measurement, t.channel)}
                  registerPdfEntry={registerPdfEntry}
                />
              ))
            )}
          </div>
        </main>
      </div>

      <footer>
        <span>CrustyPump v0.2.0</span>
      </footer>
    </div>
  );
}
