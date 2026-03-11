/** Parse "HH:MM:SS" into total seconds */
export function timeToSeconds(time: string): number {
  const [h = 0, m = 0, s = 0] = time.split(':').map(Number);
  return h * 3600 + m * 60 + s;
}

/** Format seconds since midnight as "HH:MM:SS" */
export function secondsToTime(total: number): string {
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = Math.floor(total % 60);
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Compare two "HH:MM:SS" strings for sorting */
export function compareTime(a: string, b: string): number {
  return timeToSeconds(a) - timeToSeconds(b);
}

/** Validate "HH:MM:SS" format */
export function isValidTime(time: string): boolean {
  return /^\d{2}:\d{2}:\d{2}$/.test(time) &&
    parseInt(time.slice(0, 2)) < 24 &&
    parseInt(time.slice(3, 5)) < 60 &&
    parseInt(time.slice(6, 8)) < 60;
}
