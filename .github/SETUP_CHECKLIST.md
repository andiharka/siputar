# GitHub Actions Setup Checklist

After pushing the workflows to GitHub, verify the setup:

## ✅ Pre-Push Checklist

- [ ] All workflow files exist in `.github/workflows/`
- [ ] `CONTRIBUTING.md` created
- [ ] `README.md` updated with build badge
- [ ] Dependabot configuration added

## 🚀 Post-Push Checklist

### 1. Verify Workflows Are Registered

Go to your repository on GitHub:
```
https://github.com/YOURUSERNAME/playback-announcer/actions
```

You should see:
- ✅ "Build" workflow
- ✅ "Release" workflow

### 2. Test CI Build

**First push will trigger the build:**
```bash
git push origin main
```

**Check build status:**
1. Go to Actions tab
2. Click on the "Build" workflow run
3. Verify all 3 jobs succeed:
   - ✅ build (macos-latest)
   - ✅ build (ubuntu-22.04)
   - ✅ build (windows-latest)

**If builds fail:**
- Check the logs in the failed job
- Common issues:
  - Missing dependencies (check `src-tauri/Cargo.toml`)
  - Type errors (run `bun run check` locally)
  - Rust errors (run `cargo check` locally)

### 3. Test Release Workflow

**Create a test tag:**
```bash
# Update version to 0.1.0 in:
# - package.json
# - src-tauri/Cargo.toml
# - src-tauri/tauri.conf.json

git add .
git commit -m "chore: bump version to v0.1.0"
git tag v0.1.0
git push origin main --tags
```

**Verify release:**
1. Go to: `https://github.com/YOURUSERNAME/playback-announcer/releases`
2. Check that v0.1.0 release exists
3. Verify installers are attached:
   - [ ] `.dmg` (macOS)
   - [ ] `.app.tar.gz` (macOS)
   - [ ] `.msi` (Windows)
   - [ ] `.exe` (Windows)
   - [ ] `.deb` (Linux)
   - [ ] `.AppImage` (Linux)

### 4. Test Installers

**Download and test each platform:**

**macOS:**
```bash
# Download .dmg
open playback-announcer_*.dmg
# Drag to Applications
# Right-click app → Open
```

**Windows:**
```powershell
# Download .msi
.\playback-announcer_*.msi
# Follow installer wizard
```

**Linux (Debian/Ubuntu):**
```bash
# Download .deb
sudo dpkg -i playback-announcer_*.deb
playback-announcer
```

**Linux (AppImage):**
```bash
# Download .AppImage
chmod +x playback-announcer_*.AppImage
./playback-announcer_*.AppImage
```

### 5. Verify Dependabot

**Check for Dependabot PRs:**
1. Go to: `https://github.com/YOURUSERNAME/playback-announcer/pulls`
2. Dependabot will create PRs for outdated dependencies (within a week)

**Configure Dependabot alerts:**
1. Go to: Settings → Security → Dependabot
2. Enable "Dependabot alerts" and "Dependabot security updates"

### 6. Update README Placeholders

**Replace `YOURUSERNAME` with your actual GitHub username:**

Files to update:
- [ ] `README.md` - Build badge URL
- [ ] `README.md` - Release links
- [ ] `CONTRIBUTING.md` - Repository URLs
- [ ] `.github/RELEASE.md` - Repository URLs (if applicable)

**Example:**
```markdown
# Before
![Build](https://github.com/YOURUSERNAME/playback-announcer/workflows/Build/badge.svg)

# After
![Build](https://github.com/andiharka/playback-announcer/workflows/Build/badge.svg)
```

## 🎯 Success Criteria

All items checked means setup is complete:

- [ ] Workflows visible in Actions tab
- [ ] CI builds pass on all platforms
- [ ] Release created with all installers
- [ ] All installers tested and working
- [ ] Dependabot enabled
- [ ] README placeholders updated
- [ ] Build badge shows "passing"

## 🐛 Troubleshooting

### Build Fails on macOS

**Error: "No such file or directory"**
- Check `tauri.conf.json` has correct icon paths

**Error: "rustc error"**
- Update Rust: `rustup update`

### Build Fails on Windows

**Error: "missing MSVC"**
- Add to workflow: `uses: microsoft/setup-msbuild@v1`

**Error: "WebView2 not found"**
- This is expected in CI, should be bundled automatically

### Build Fails on Linux

**Error: "package not found"**
- Add missing packages to `apt-get install` in workflow

### Release Not Created

**Error: "Resource not accessible by integration"**
- Check repository settings → Actions → Workflow permissions
- Set to: "Read and write permissions"

### Artifacts Not Uploaded

**Error: "No files matching pattern"**
- Check `target/release/bundle/` path in workflow
- Verify Tauri is building the correct formats

## 📚 Resources

- [Tauri Actions Documentation](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [Semantic Versioning](https://semver.org/)

## 🎉 Next Steps

Once everything is verified:

1. **Announce the release** - Share with users
2. **Set up auto-updates** - Configure Tauri updater (optional)
3. **Add code signing** - Eliminate security warnings (optional)
4. **Create roadmap** - Plan future releases

---

**Last Updated:** March 2024
