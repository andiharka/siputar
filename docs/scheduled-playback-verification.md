# Scheduled playback reliability

The Rust scheduler checks saved, enabled schedules every second against local
wall-clock time and active weekdays. It catches delayed ticks within ten seconds,
including ticks that cross midnight. Older missed occurrences (for example, after
sleep) are intentionally skipped. The computer must be awake and SIPUTAR running.

Playback is coordinated once per main webview by `src/lib/scheduledPlayback.ts`,
initialized from the persistent app layout. Navigation between Schedules and Audio
neither removes the scheduler listener nor reloads configuration. All playback
listeners are installed before saved schedules are sent to Rust. Scheduled events
use the saved playlist; the manual Play button can preview unsaved edits.

Windows-specific protections:

- `open_mini_player` is asynchronous, avoiding the documented WebView2 deadlock
  when its fallback creates a window from a synchronous Tauri command.
- Loading/saving schedules restores asset access for their individual media files,
  including files on other drives and UNC shares. Dialog grants alone do not
  survive restarts.
- The mini-player readiness handshake allows up to 15 seconds for initialization;
  playback begins immediately on a matching response.
- Optional Web Audio warm-up waits at most 500 ms for a suspended context to resume,
  so a pending resume cannot indefinitely block HTML media playback.
- The existing Windows autoplay argument remains in place.

A “Playback started” activity entry with a schedule ID is now written after the
media element's `play()` succeeds. Opening/loading failures are recorded as
“Playback failed” with their details. This confirms the media engine started;
muted system output or disconnected hardware still requires an audible check.

## Automated checks

```sh
bun run test
bun run check
bun run build
cargo test --manifest-path src-tauri/Cargo.toml --lib scheduler::tests
```

The Build workflow runs playback and scheduler tests on Windows, macOS and Linux.
Frontend tests mock Tauri IPC and audio contexts; they do not replace a real
WebView2/audio-device test. Rust tests cover due times, late ticks, midnight/day
selection, invalid/disabled schedules, and bounded sleep/clock-change behavior.

## Windows device acceptance checks

Run these on an installed Windows build; they cannot be validated by the macOS
workspace's automated tests:

1. Save a schedule one minute ahead using a WAV/MP3 outside the user profile
   (for example `D:\Announcements\bell.wav`). Restart SIPUTAR without reselecting
   the file. Verify playback and a successful activity entry at the saved time.
2. Repeat from the Audio page, with the main window minimized, and with it closed
   to the tray. Keep Windows awake and do not click Play before the scheduled time.
3. Use two tracks with track and schedule repeats. Navigate between pages during
   playback; verify one active queue, correct repeats, Pause/Resume, Stop and Reset.
4. Edit the playlist without saving. Verify scheduled playback uses the last saved
   version, while manual Play previews the edits.
5. Leave audio idle, then schedule another announcement. Verify its opening audio
   is audible on the intended output device. Repeat after reconnecting that device.
6. Test a missing file and confirm a failure entry, queue progression, and successful
   playback of a later valid schedule. Verify Stop during startup cannot cause a
   delayed start after the stop request.
7. For network media, repeat after restarting with the share mounted and accessible.
   Disconnected shares and unsupported codecs are expected to report media failures.

References: [Tauri window creation and Windows deadlocks](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html),
[Tauri asset protocol scope](https://docs.rs/tauri/latest/tauri/trait.Manager.html#method.asset_protocol_scope).
