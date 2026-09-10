let warmupCtx: AudioContext | null = null;
let warmingPromise: Promise<void> | null = null;

/**
 * Returns a shared AudioContext singleton for audio pre-warming.
 */
export function getWarmupAudioContext(): AudioContext | null {
  if (typeof window === 'undefined') return null;
  if (!warmupCtx) {
    const AudioContextClass =
      window.AudioContext ||
      (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (AudioContextClass) {
      try {
        warmupCtx = new AudioContextClass();
      } catch (e) {
        console.warn('[AudioWarmup] Could not initialize AudioContext:', e);
      }
    }
  }
  return warmupCtx;
}

/**
 * Pre-warms the Windows audio engine (WASAPI) and hardware DAC by playing
 * a tiny silent buffer through Web Audio API.
 *
 * This prevents the first 2-3 seconds of audible blank/cut-off on Windows,
 * which occurs when an idle audio endpoint wakes up from low-power state.
 */
export async function warmupAudioDevice(): Promise<void> {
  // If a warmup is already in flight, reuse its promise
  if (warmingPromise) {
    return warmingPromise;
  }

  warmingPromise = (async () => {
    try {
      const ctx = getWarmupAudioContext();
      if (!ctx) return;

      if (ctx.state === 'suspended') {
        await ctx.resume();
      }

      // Play 50ms of silence through the output destination to open the WASAPI stream
      const sampleRate = ctx.sampleRate || 44100;
      const frameCount = Math.max(1, Math.floor(sampleRate * 0.05));
      const buffer = ctx.createBuffer(1, frameCount, sampleRate);
      
      const source = ctx.createBufferSource();
      source.buffer = buffer;
      source.connect(ctx.destination);
      source.start(0);

      // Give the hardware DAC and WASAPI a brief window (~150ms) to complete the handshake
      await new Promise((resolve) => setTimeout(resolve, 150));
      console.log('[AudioWarmup] Audio device pre-warmed successfully (state:', ctx.state, ')');
    } catch (err) {
      console.warn('[AudioWarmup] Pre-warmup failed (continuing gracefully):', err);
    } finally {
      warmingPromise = null;
    }
  })();

  return warmingPromise;
}
