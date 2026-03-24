# Quick Release Guide

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

### 3. Create and push tag:

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

### 4. Wait for GitHub Actions to complete

GitHub Actions will automatically:
- Create GitHub Release
- Build for macOS, Windows, Linux
- Upload installers (.dmg, .msi, .deb, .AppImage, etc.)

### 5. Edit release notes on GitHub

Add:
- What's New
- Bug Fixes
- Breaking Changes (if any)

### 6. Publish the release

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
