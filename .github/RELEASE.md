# Quick Release Guide

## ⚡ One-Time Setup (First Release Only)

### 1. Generate signing keys:
```bash
bunx tauri signer generate -w ~/.tauri/siputar.key
```
This will print your **public key** in the terminal.

### 2. Paste the public key into `src-tauri/tauri.conf.json`:
```json
"plugins": {
  "updater": {
    "pubkey": "<PASTE_PUBLIC_KEY_HERE>",
    ...
  }
}
```

### 3. Add GitHub Secrets (repo → Settings → Secrets → Actions):
- `TAURI_SIGNING_PRIVATE_KEY` — contents of `~/.tauri/siputar.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — leave empty (or your chosen password)

---

## Steps to Create a New Release

### 1. Update version in 3 files:

```bash
# package.json
{
  "version": "X.Y.Z"
}

# src-tauri/Cargo.toml
[package]
version = "X.Y.Z"

# src-tauri/tauri.conf.json
{
  "version": "X.Y.Z"
}
```

### 2. Commit and push:

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to vX.Y.Z"
git push origin main
```

Let the main-branch Build workflow pass before pushing the release tag. It fills
the shared per-platform Rust cache, which the Release workflow can reuse.
Starting both workflows together on a cold cache makes both compile dependencies.

### 3. Create and push tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

### 4. Wait for GitHub Actions to complete

GitHub Actions will automatically:
- Create GitHub Release
- Build for macOS, Windows, Linux
- Sign all bundles with your private key
- Upload installers (.dmg, .msi, .deb, .AppImage, etc.)
- Generate and upload `latest.json` for in-app updates

### 5. Edit release notes on GitHub

Add:
- What's New
- Bug Fixes
- Breaking Changes (if any)

### 6. Publish the release

Existing installs will detect the new version automatically on next launch (3-second delayed check), or via Settings → About & Updates.

## Build time and caching

The [v0.7.2 release](https://github.com/andiharka/siputar/actions/runs/34437045299)
had no Rust cache hits on any platform. Rust compilation alone took 4m25s on
macOS, 6m17s on Ubuntu, and 9m24s on Windows; packaging and setup added more time.
These are cold-build timings, not the expected cost of every code change.

Build and Release now share release-dependency caches per platform, including
dependencies compiled during failed runs. Scheduler tests use the release profile
and Tauri's `custom-protocol` feature after the app build, avoiding a separate
debug dependency compilation. Superseded branch/PR builds are canceled.

The first run after changing the cache key or Rust toolchain is still cold.
Dependency changes may also need recompilation. Compare subsequent runs with the
same toolchain and lockfiles before judging the improvement. Installers and tests
remain enabled on all three platforms.

CI uses `bun install --frozen-lockfile` and Cargo's `--locked`. When updating Tauri,
keep the Rust and JavaScript packages on matching major/minor versions and commit
both lockfiles. A version mismatch prevents packaging even if Rust tests pass.

## Version Numbering (SemVer)

- **Major (1.0.0)** - Breaking changes
- **Minor (0.1.0)** - New features
- **Patch (0.0.1)** - Bug fixes

## Pre-releases

For beta/RC releases:

```bash
git tag vX.Y.Z-beta.1
git push origin vX.Y.Z-beta.1
```

Mark as "pre-release" on GitHub.
