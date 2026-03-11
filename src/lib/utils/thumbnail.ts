import { convertFileSrc } from '@tauri-apps/api/core';
export function getMediaType(path: string): 'video' | 'audio' {
  const ext = path.split('.').pop()?.toLowerCase() ?? '';
  const videoExts = ['mp4', 'mkv', 'avi', 'mov', 'webm', 'wmv', 'flv', 'm4v'];
  return videoExts.includes(ext) ? 'video' : 'audio';
}

/** Extract file name from full path */
export function getFileName(path: string): string {
  return path.replace(/\\/g, '/').split('/').pop() ?? path;
}

/** Cache: mediaId → data URL */
const thumbnailCache = new Map<string, string>();

/** Generate a thumbnail for a video file (first frame) via an offscreen canvas */
export async function getVideoThumbnail(mediaId: string, filePath: string): Promise<string | null> {
  if (thumbnailCache.has(mediaId)) return thumbnailCache.get(mediaId)!;

  return new Promise(resolve => {
    const video = document.createElement('video');
    video.preload = 'metadata';
    video.muted = true;
    video.crossOrigin = 'anonymous';

    video.addEventListener('loadeddata', () => {
      video.currentTime = 0.5;
    });

    video.addEventListener('seeked', () => {
      try {
        const canvas = document.createElement('canvas');
        canvas.width = 120;
        canvas.height = 68;
        const ctx = canvas.getContext('2d');
        if (!ctx) { resolve(null); return; }
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
        const dataUrl = canvas.toDataURL('image/jpeg', 0.7);
        thumbnailCache.set(mediaId, dataUrl);
        resolve(dataUrl);
      } catch {
        resolve(null);
      } finally {
        video.src = '';
        video.load();
      }
    }, { once: true });

    video.addEventListener('error', () => resolve(null), { once: true });
    video.src = convertFileSrc(filePath);
  });
}

/** Cache: mediaId → duration in seconds */
const durationCache = new Map<string, number>();

/** Load media metadata to get duration in seconds. Cached by mediaId. */
export async function getMediaDuration(mediaId: string, filePath: string, type: 'video' | 'audio'): Promise<number> {
  if (durationCache.has(mediaId)) return durationCache.get(mediaId)!;

  return new Promise(resolve => {
    const el = document.createElement(type);
    el.preload = 'metadata';
    if (type === 'video') (el as HTMLVideoElement).muted = true;

    el.addEventListener('loadedmetadata', () => {
      const dur = isFinite(el.duration) ? el.duration : 0;
      durationCache.set(mediaId, dur);
      el.src = '';
      el.load();
      resolve(dur);
    }, { once: true });

    el.addEventListener('error', () => { el.src = ''; resolve(0); }, { once: true });
    el.src = convertFileSrc(filePath);
  });
}

export function clearDurationCache(): void {
  durationCache.clear();
}
