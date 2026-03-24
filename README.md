# Playback Announcer

A desktop application for scheduling and playing multimedia announcements with Text-to-Speech (TTS) integration using ElevenLabs.

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![License](https://img.shields.io/badge/license-GPL--3.0-green.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)
![Svelte](https://img.shields.io/badge/Svelte-5.0-red.svg)
![Build](https://github.com/andiharka/playback-announcer/workflows/Build/badge.svg)

## 📋 Overview

Playback Announcer is a cross-platform desktop application designed for scheduling and playing audio/video announcements at specific times. It features a built-in Text-to-Speech system powered by ElevenLabs, allowing you to generate professional voice announcements directly within the app.

### Key Features

- **⏰ Schedule Management**: Create, edit, and manage multiple time-based playback schedules
- **🎵 Multi-Media Support**: Play audio (MP3, WAV, etc.) and video files
- **🔁 Loop Control**: Configure loop counts for individual media files
- **📢 ElevenLabs TTS Integration**: Generate high-quality voice announcements with customizable settings
- **🎛️ Voice Customization**: Adjust stability, similarity boost, and speed for TTS generation
- **💾 Offline Storage**: Access previously generated audio files without internet connection
- **📊 Quota Tracking**: Monitor your ElevenLabs character usage and remaining credits
- **🎨 Dark/Light Themes**: Automatic theme switching based on system preferences
- **🪟 Mini-Player**: Floating always-on-top player window for convenient playback monitoring
- **🔐 Secure API Key Storage**: API keys stored securely in system keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)

## 🚀 Getting Started

### Prerequisites

- **Node.js** (v20 or higher) and **Bun** package manager
- **Rust** (latest stable version)
- **macOS**, **Windows**, or **Linux** operating system

### Installation

#### Option 1: Install from Release (Recommended)

1. Go to the [Releases](https://github.com/andiharka/playback-announcer/releases) page and download the latest version
2. Choose the installer for your platform:
   - **macOS**: Download the `.dmg` file
     - Open the `.dmg` file
     - Drag the app to Applications folder
     - **Important:** GitHub-built apps are unsigned and will show "damaged" error
     - **Fix:** Open Terminal and run:
       ```bash
       xattr -cr "/Applications/Playback Announcer.app"
       ```
     - Then double-click the app to launch
     - See [Troubleshooting](#troubleshooting) for details
   - **Windows**: Download the `.msi` installer
     - Run the `.msi` installer
     - Follow the installation wizard
     - Windows SmartScreen may appear (click "More info" → "Run anyway" for unsigned builds)
   - **Linux**: Download the `.deb` (Debian/Ubuntu) or `.AppImage` (universal)
     - **Debian/Ubuntu**: `sudo dpkg -i playback-announcer_*.deb`
     - **AppImage**: Make executable (`chmod +x *.AppImage`) then run

> **Note:** Builds are currently unsigned. macOS and Windows will show security warnings. This is normal for open-source apps without paid developer certificates.

#### Option 2: Build from Source

1. **Clone the repository**
   ```bash
   git clone https://github.com/andiharka/playback-announcer.git
   cd playback-announcer
   ```

2. **Install dependencies**
   ```bash
   bun install
   ```

3. **Run in development mode**
   ```bash
   bun tauri dev
   ```

4. **Build for production**
   ```bash
   bun tauri build
   ```

## 📖 How to Use

### 1. Initial Setup

**Configure Settings:**
- Click the **Settings** ⚙️ icon in the top-right corner
- Set your preferred theme (Light/Dark/Auto)
- Enable "Run on Startup" if desired
- Configure the audio folder path for TTS-generated files

**Add ElevenLabs API Key (Optional for TTS):**
1. Go to Settings → ElevenLabs TTS
2. Enter your API key (get one from [ElevenLabs](https://elevenlabs.io))
3. Click "Test Connection" to verify
4. The key is securely stored in your system keychain

### 2. Creating Schedules

1. Navigate to the **Schedule List** tab
2. Click **Add Schedule** ➕ button in the header
3. Configure the schedule:
   - **Name**: Give your schedule a descriptive name
   - **Time**: Set when the announcement should play (24-hour format)
   - **Days**: Select which days the schedule is active
   - **Media Files**: Add audio/video files to play
   - **Volume**: Adjust volume for each file (0-100%)
   - **Loop Count**: Set how many times to repeat each file

4. **Add Media Files:**
   - Click "Add Media" in the schedule card
   - Browse and select your audio/video files
   - Drag to reorder files in the playlist
   - Set individual volume and loop settings

5. Click **Save** to apply changes

### 3. Managing Playback

**Manual Playback:**
- Click the ▶️ **Play** button on any schedule card to test it immediately
- Use the mini-player window to control playback (Pause/Stop)

**Automatic Scheduling:**
- Click **Resume All** to activate all scheduled announcements
- Schedules will automatically play at their configured times
- Click **Pause All** to temporarily disable all schedules

**Mini-Player Window:**
- Opens automatically when playback starts
- Always-on-top floating window
- Shows current media, status, and playlist
- Drag anywhere to reposition

### 4. Generating TTS Audio

1. Navigate to the **Generate Audio** tab
2. Click **Generate Audio** ➕ button
3. Fill in the TTS form:
   - **Text**: Enter your announcement text
   - **Voice**: Select from available ElevenLabs voices
   - **Model**: Choose the TTS model
   - **Language**: Select language (default: Indonesian)
   - **Speed**: Adjust speaking speed (if supported)
   - **Stability**: Control voice consistency
   - **Similarity**: Adjust voice similarity to original

4. Click **Generate**
5. The audio appears in the history list once generated
6. Click **Play** to preview or **Open Folder** to locate the file

**Using Generated Audio in Schedules:**
- Generated files are saved to your configured audio folder
- Add them to schedules like any other audio file
- Files remain accessible offline after generation

### 5. Managing TTS History

**View History:**
- All generated audio appears in the history list
- Shows text snippet, date, voice, model, and character count

**Actions:**
- **Play/Stop**: Preview the audio
- **Open Folder**: Navigate to the file location
- **Download**: Re-download if local file is missing
- **Delete**: Remove from history and delete local file

**Sync with ElevenLabs:**
- Click **Sync** to refresh history from ElevenLabs
- Syncing merges remote history with local database
- Works offline with locally stored data

## 🛠️ Development

### Tech Stack

- **Frontend**: SvelteKit 5 + Svelte 5 (with runes)
- **Backend**: Tauri 2 (Rust)
- **Database**: SQLite (via tauri-plugin-sql)
- **Styling**: Custom CSS with CSS variables
- **Icons**: Tabler Icons
- **TTS**: ElevenLabs API

### Project Structure

```
playback-announcer/
├── src/                    # Frontend source
│   ├── lib/
│   │   ├── components/    # Svelte components
│   │   ├── stores/        # State management
│   │   ├── types/         # TypeScript types
│   │   ├── utils/         # Helper functions
│   │   └── i18n/          # Translations (ID/EN)
│   ├── routes/            # SvelteKit routes
│   │   ├── app/           # Main app routes
│   │   │   ├── schedules/ # Schedule list page
│   │   │   └── audio/     # TTS audio page
│   │   └── mini-player/   # Mini-player window
│   └── app.css            # Global styles
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands.rs    # Tauri commands
│   │   ├── keychain.rs    # System keychain access
│   │   ├── elevenlabs.rs  # ElevenLabs API client
│   │   └── lib.rs         # Main entry point
│   └── Cargo.toml         # Rust dependencies
└── static/                 # Static assets
```

### Available Scripts

```bash
# Development
bun tauri dev              # Run in development mode
bun run check              # Type-check TypeScript/Svelte
bun run check:watch        # Type-check in watch mode

# Building
bun tauri build            # Build for production

# Testing
cargo test --manifest-path src-tauri/Cargo.toml  # Run Rust tests
```

### Key Technologies & Patterns

**State Management:**
- Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Reactive stores with `.svelte.ts` extension

**Secure Storage:**
- API keys stored via system keychain (keyring crate)
- Supports macOS, Windows, and Linux

**Database:**
- SQLite for schedule and TTS history persistence
- Tauri plugin for async database access

**Rate Limiting:**
- 1 request/second for ElevenLabs API
- Prevents triggering unusual activity flags

### Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines and development setup.

## 🔧 Troubleshooting

### macOS: "App is damaged and can't be opened"

This error occurs because GitHub-built apps are not code-signed with an Apple Developer certificate.

**Solution:**

1. Open Terminal
2. Run this command to remove the quarantine flag:
   ```bash
   xattr -cr "/Applications/Playback Announcer.app"
   ```
3. The app will now open normally

**Why this happens:**
- macOS Gatekeeper blocks unsigned apps downloaded from the internet
- Apps built locally don't have this issue
- Right-click → Open doesn't work for unsigned GitHub-built apps
- The `xattr -cr` command removes the quarantine attribute, allowing the app to run

**Alternative:** Build from source locally (see [Build from Source](#option-2-build-from-source))

### Windows: SmartScreen Warning

Windows may show "Windows protected your PC" when running the installer.

**Solution:**
1. Click "More info"
2. Click "Run anyway"

This is normal for unsigned applications. The app is safe to run.

### Linux: Permission Denied (AppImage)

If the AppImage won't run:

```bash
chmod +x playback-announcer_*.AppImage
./playback-announcer_*.AppImage
```

### ElevenLabs API Key Issues

**"API key not configured" error:**

1. Go to Settings → ElevenLabs section
2. Paste your API key
3. Click "Test Connection"
4. If test succeeds but error persists, restart the app

**On first macOS launch:**
- Keychain may prompt for permission
- Click "Always Allow" for best experience

### Database Errors

If you encounter database errors, the SQLite file may be corrupted:

**macOS:** `~/Library/Application Support/com.disperpusip.playbackannouncer/`
**Windows:** `%APPDATA%\com.disperpusip.playbackannouncer\`
**Linux:** `~/.local/share/com.disperpusip.playbackannouncer/`

Delete the `playback_announcer.db` file and restart the app.

## 📄 License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

### What this means:

- ✅ You can use this software for any purpose
- ✅ You can modify the source code
- ✅ You can distribute the software
- ✅ You can distribute modified versions
- ⚠️ **You must disclose the source code** when distributing
- ⚠️ **You must license derivative works under GPL-3.0**
- ⚠️ **You must state significant changes** made to the software
- ⚠️ **You must include the original copyright notice**

This is a **copyleft license**, meaning any derivative work must also be open-source under the same license.

See the [LICENSE](LICENSE) file for the full license text or visit [https://www.gnu.org/licenses/gpl-3.0.html](https://www.gnu.org/licenses/gpl-3.0.html)

## 🙏 Acknowledgments

- **Tauri** - For the amazing cross-platform framework
- **SvelteKit** - For the excellent web framework
- **ElevenLabs** - For the powerful Text-to-Speech API
- **Tabler Icons** - For the beautiful icon set

## 📞 Support

For issues, questions, or feature requests:
- Open an issue on [GitHub Issues](https://github.com/andiharka/playback-announcer/issues)
- Check existing issues before creating a new one

## 🗺️ Roadmap

- [ ] Multi-language support expansion
- [ ] Custom notification sounds
- [ ] Schedule templates
- [ ] Export/import schedules
- [ ] Audio waveform visualization
- [ ] More TTS provider integrations

---

**Made with ❤️ for Disperpusip Jawa Timur**

© 2026 Andi. All rights reserved.
