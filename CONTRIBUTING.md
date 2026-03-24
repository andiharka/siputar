# Contributing to Playback Announcer

Thank you for your interest in contributing to Playback Announcer! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Release Process](#release-process)
- [Coding Standards](#coding-standards)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/YOURUSERNAME/playback-announcer/issues)
2. If not, create a new issue with:
   - Clear, descriptive title
   - Steps to reproduce
   - Expected vs actual behavior
   - Screenshots if applicable
   - Your environment (OS, version)

### Suggesting Features

1. Check if the feature has been requested in [Issues](https://github.com/YOURUSERNAME/playback-announcer/issues)
2. Create a new issue with:
   - Clear description of the feature
   - Use cases and benefits
   - Possible implementation approach (optional)

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following the coding standards
4. Test your changes thoroughly
5. Commit with clear messages (`git commit -m 'Add amazing feature'`)
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request with:
   - Description of changes
   - Related issue number (if applicable)
   - Screenshots/demos if UI changes

## Development Setup

### Prerequisites

- **Node.js** v20 or higher (v18 is not supported due to missing `styleText` in node:util)
- **Bun** package manager
- **Rust** (latest stable)
- **Platform-specific dependencies:**
  - **Linux**: `libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft C++ Build Tools

### Setup Steps

1. **Clone your fork**
   ```bash
   git clone https://github.com/YOURUSERNAME/playback-announcer.git
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

4. **Run type checking**
   ```bash
   bun run check
   ```

5. **Run tests (Rust)**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

## Making Changes

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes

### Commit Messages

Follow conventional commits format:

```
type(scope): description

[optional body]
[optional footer]
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation
- `style` - Formatting, missing semicolons, etc.
- `refactor` - Code restructuring
- `test` - Adding tests
- `chore` - Maintenance tasks

**Examples:**
```
feat(tts): add voice speed control slider
fix(scheduler): resolve timing issue with daylight savings
docs(readme): update installation instructions
```

### Testing

Before submitting a PR:

1. **Type check passes:**
   ```bash
   bun run check
   ```

2. **Rust tests pass:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

3. **App builds successfully:**
   ```bash
   bun tauri build
   ```

4. **Manual testing:**
   - Test on your platform
   - Verify all affected features work
   - Check for console errors

## Release Process

### For Maintainers

Releases are automated through GitHub Actions. To create a new release:

### 1. Update Version Numbers

Update version in three files:

**package.json:**
```json
{
  "version": "0.2.0"
}
```

**src-tauri/Cargo.toml:**
```toml
[package]
version = "0.2.0"
```

**src-tauri/tauri.conf.json:**
```json
{
  "version": "0.2.0"
}
```

### 2. Commit Version Bump

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to v0.2.0"
git push origin main
```

### 3. Create and Push Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. Automated Build Process

Once the tag is pushed, GitHub Actions will automatically:

1. ✅ Create a GitHub Release
2. ✅ Build for macOS, Windows, and Linux in parallel
3. ✅ Generate platform-specific installers:
   - **macOS**: `.dmg`, `.app.tar.gz`
   - **Windows**: `.msi`, `.exe`
   - **Linux**: `.deb`, `.AppImage`
4. ✅ Upload installers as release assets
5. ✅ Generate release notes

### 5. Publish Release

1. Go to [Releases](https://github.com/YOURUSERNAME/playback-announcer/releases)
2. Find the newly created draft release
3. Edit the release notes to add:
   - **What's New** - Major features and improvements
   - **Bug Fixes** - Issues resolved
   - **Breaking Changes** - If any
   - **Contributors** - Thank contributors
4. Click "Publish release"

### Version Numbering (Semantic Versioning)

Follow [SemVer](https://semver.org/):

- **Major (1.0.0)** - Breaking changes
- **Minor (0.1.0)** - New features (backward compatible)
- **Patch (0.0.1)** - Bug fixes (backward compatible)

**Examples:**
- `0.1.0` → `0.2.0` - Added TTS feature
- `0.2.0` → `0.2.1` - Fixed TTS bug
- `0.9.0` → `1.0.0` - First stable release

### Pre-releases

For testing releases before making them public:

```bash
git tag v0.2.0-beta.1
git push origin v0.2.0-beta.1
```

Mark the release as "pre-release" in GitHub.

## Coding Standards

### TypeScript/Svelte

- Use Svelte 5 runes (`$state`, `$derived`, `$effect`)
- Follow existing patterns in the codebase
- Prefer functional over imperative style
- Use TypeScript strict mode
- Comment complex logic only

### Rust

- Follow Rust conventions (snake_case for functions/variables)
- Use `rustfmt` for formatting
- Avoid `unwrap()` in production code (use proper error handling)
- Add doc comments for public functions

### CSS

- Use CSS variables for theming
- Follow BEM-like naming for components
- Keep styles scoped to components
- Support both light and dark themes

### File Structure

```
src/
├── lib/
│   ├── components/    # Svelte components
│   ├── stores/        # State management (.svelte.ts)
│   ├── types/         # TypeScript types
│   ├── utils/         # Helper functions
│   └── i18n/          # Translations
├── routes/            # SvelteKit routes
│   ├── app/           # Main app pages
│   └── mini-player/   # Mini-player window
└── app.css            # Global styles

src-tauri/
└── src/
    ├── commands.rs    # Tauri commands
    ├── keychain.rs    # Keychain access
    ├── elevenlabs.rs  # API client
    └── lib.rs         # Entry point
```

## Questions?

If you have questions about contributing:

1. Check existing [Issues](https://github.com/YOURUSERNAME/playback-announcer/issues)
2. Open a new issue with the "question" label
3. Reach out to the maintainers

Thank you for contributing! 🎉
