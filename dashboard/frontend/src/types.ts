export type TimeRangePreset = '1m' | '2m' | '15m' | '1h' | '24h' | '7d' | 'custom';

export interface Channel {
  channel: string;
  measurement: string;
}

export interface TileConfig {
  channel: string;
  measurement: string;
}

export interface DashboardConfig {
  time_range_preset?: TimeRangePreset;
  time_range?: {
    start: string;
    end: string;
  };
  granularity?: string;
  tiles?: TileConfig[];
}

export interface DataPoint {
  bucket: string;
  val: number | null;
}
