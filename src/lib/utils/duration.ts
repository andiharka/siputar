/** Format seconds as "Xj Ym Zd" (id) or "Xh Ym Zs" (en) */
export function formatDuration(seconds: number, lang: 'id' | 'en' = 'id'): string {
  if (!isFinite(seconds) || seconds <= 0) return '—';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  const parts: string[] = [];
  if (h > 0) parts.push(lang === 'id' ? `${h}j` : `${h}h`);
  if (m > 0) parts.push(`${m}m`);
  if (s > 0 || parts.length === 0) parts.push(lang === 'id' ? `${s}d` : `${s}s`);
  return parts.join(' ');
}

/** Calculate total playback seconds for a list of media items (with loop counts) */
export function calcTotalDuration(durations: Map<string, number>, media: { id: string; loopCount: number }[]): number {
  return media.reduce((acc, m) => {
    const d = durations.get(m.id) ?? 0;
    return acc + d * Math.max(1, m.loopCount);
  }, 0);
}
