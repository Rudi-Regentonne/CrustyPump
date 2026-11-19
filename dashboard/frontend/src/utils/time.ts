import type { TimeRangePreset } from '../types';

export function localInputToUtcIso(value: string): string {
  if (!value) return value;
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toISOString();
}

export function toLocalISOString(date: Date): string {
  const tzOffset = date.getTimezoneOffset() * 60000;
  return new Date(date.getTime() - tzOffset).toISOString().slice(0, -5);
}

export function getStartEndDates(
  preset: TimeRangePreset,
  customStart: string,
  customEnd: string,
): { start: string; end: string } {
  if (preset === 'custom') {
    return {
      start: localInputToUtcIso(customStart),
      end: localInputToUtcIso(customEnd),
    };
  }

  const end = new Date();
  const start = new Date(end);

  switch (preset) {
    case '1m':
      start.setMinutes(start.getMinutes() - 1);
      break;
    case '2m':
      start.setMinutes(start.getMinutes() - 2);
      break;
    case '15m':
      start.setMinutes(start.getMinutes() - 15);
      break;
    case '1h':
      start.setHours(start.getHours() - 1);
      break;
    case '24h':
      start.setHours(start.getHours() - 24);
      break;
    case '7d':
      start.setDate(start.getDate() - 7);
      break;
  }

  return {
    start: start.toISOString(),
    end: end.toISOString(),
  };
}
