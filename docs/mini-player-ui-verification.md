# Mini player UI verification

Verified on October 29, 2026.

## Scope

The player uses SIPUTAR's theme tokens, separate audio and video presentations,
an accessible seek slider, a fixed transport area, and a scrolling queue. Both
the configured window and fallback window use a 380 × 520 content size with
native window decorations. The separate player webview loads display
preferences without initializing or updating the scheduler.

## Browser verification

A temporary Svelte route mounted the production MiniPlayer component. Tauri IPC
was mocked using the installed Tauri mock utilities and an event dispatcher.
Media playback used a real, muted, 180-second WAV and a generated 640 × 360
video. The route and media fixtures were removed before the production build.

The following checks passed:

- Audio, video, and idle layouts at 380 × 520; controls and the first queue row
  also fit at 380 × 480 without horizontal overflow.
- Long current-track filenames wrap to two lines; queue filenames truncate.
- Pause/resume changes the actual media element's paused state.
- Arrow keys seek by five seconds, Page Up advances by 10%, Home seeks to zero,
  and End clamps to the media's duration.
- Pausing during media loading leaves playback paused at time zero after
  loading finishes. Resuming starts it successfully.
- Repeated pause/resume produces only one playback-start acknowledgement per
  track. Restart emits the existing schedule-reset event.
- Stop clears the media state and queue, then disables the transport controls.
- A failed file produces one completion/error event and dismissible feedback
  that remains visible when the next track starts.
- Finite and infinite schedule repeat information renders correctly.
- An 11-item queue scrolls the last active item fully into view.
- Live theme and language changes update the player during playback.
- A live dark/Indonesian preference update takes precedence over a delayed
  light/English disk response.
- Audio and video elements remain present together when switching media types.

## Automated checks

```sh
bun run check
bun test tests
bun run build
cargo check --manifest-path src-tauri/Cargo.toml
```

The existing playback tests cover queue advancement, repetitions, stale events,
window readiness, schedule restart, and audio warm-up behavior.

## Remaining native QA

Browser verification uses mocked IPC. Test the packaged app on Windows WebView2
and macOS for native window dragging, hide/reopen, display scaling, live
cross-window preference delivery, and audio-device output. The Rust check
validates the host build, not the Windows build or these native interactions.
