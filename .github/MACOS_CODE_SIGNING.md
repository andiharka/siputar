# macOS Code Signing Guide (Optional)

This guide explains how to set up proper code signing for macOS builds to eliminate the "damaged app" error.

## Current Status

**Unsigned builds** - GitHub Actions builds are not code-signed, resulting in:
- ❌ macOS Gatekeeper shows "damaged and can't be opened" error
- ✅ Workaround: Users run `xattr -cr "/Applications/Playback Announcer.app"`
- ✅ App functions perfectly after removing quarantine flag

## Why Ad-hoc Signing Doesn't Help

Ad-hoc signing (`codesign --sign -`) was considered but **does not solve the quarantine issue**:

1. **Quarantine flag is applied on download** - Not during build
2. **Ad-hoc signatures are not trusted** - Gatekeeper still blocks
3. **Right-click → Open still fails** - Same as unsigned
4. **Users still need `xattr -cr`** - No improvement

**Conclusion:** Only proper signing with Apple Developer certificate eliminates the error.

## Proper Code Signing Setup

### Requirements

1. **Apple Developer Program** membership ($99/year)
   - Sign up at: https://developer.apple.com/programs/

2. **Developer ID Application Certificate**
   - Obtained from Apple Developer Portal
   - Allows distribution outside Mac App Store

3. **App-specific Password** (for notarization)
   - Created in Apple ID account settings

### Step 1: Generate Certificate

1. Go to https://developer.apple.com/account/resources/certificates/list
2. Click "+" to create new certificate
3. Select "Developer ID Application"
4. Follow instructions to generate CSR (Certificate Signing Request)
5. Download certificate (.cer file)
6. Import to Keychain Access on Mac

### Step 2: Export Certificate for CI/CD

```bash
# Export certificate with private key
security find-identity -v -p codesigning

# Export as .p12 (you'll set a password)
# In Keychain Access: Right-click certificate → Export

# Convert to base64 for GitHub Secrets
base64 -i certificate.p12 | pbcopy
```

### Step 3: Set Up GitHub Secrets

Go to GitHub repository → Settings → Secrets and variables → Actions

Add these secrets:

1. **MACOS_CERTIFICATE**
   - The base64-encoded .p12 certificate (from step 2)

2. **MACOS_CERTIFICATE_PWD**
   - Password you set when exporting the .p12

3. **APPLE_ID**
   - Your Apple ID email (for notarization)

4. **APPLE_APP_PASSWORD**
   - App-specific password from appleid.apple.com

5. **APPLE_TEAM_ID**
   - Found in Apple Developer account (10-character ID)

### Step 4: Update GitHub Workflows

Add these steps to `.github/workflows/build.yml` and `.github/workflows/release.yml` (macOS jobs only):

```yaml
      - name: Import code signing certificate (macOS)
        if: matrix.platform == 'macos-latest'
        run: |
          # Create temporary keychain
          security create-keychain -p actions build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p actions build.keychain
          security set-keychain-settings -t 3600 -u build.keychain
          
          # Import certificate
          echo "${{ secrets.MACOS_CERTIFICATE }}" | base64 --decode > certificate.p12
          security import certificate.p12 -k build.keychain -P "${{ secrets.MACOS_CERTIFICATE_PWD }}" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k actions build.keychain
          
          # Verify
          security find-identity -v -p codesigning

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.MACOS_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.MACOS_CERTIFICATE_PWD }}
          APPLE_SIGNING_IDENTITY: "Developer ID Application: YOUR NAME (TEAM_ID)"
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          args: --verbose

      - name: Notarize macOS app (macOS)
        if: matrix.platform == 'macos-latest'
        run: |
          # Find the DMG
          DMG_PATH=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" | head -1)
          
          # Submit for notarization
          xcrun notarytool submit "$DMG_PATH" \
            --apple-id "${{ secrets.APPLE_ID }}" \
            --password "${{ secrets.APPLE_APP_PASSWORD }}" \
            --team-id "${{ secrets.APPLE_TEAM_ID }}" \
            --wait
          
          # Staple notarization ticket
          xcrun stapler staple "$DMG_PATH"
          
          echo "Notarization complete!"

      - name: Cleanup keychain (macOS)
        if: matrix.platform == 'macos-latest' && always()
        run: |
          security delete-keychain build.keychain
```

### Step 5: Update tauri.conf.json

Add signing configuration:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "macOS": {
      "signingIdentity": null,
      "hardenedRuntime": true,
      "entitlements": null
    }
  }
}
```

### Step 6: Test

1. Push changes to trigger build
2. Download built `.dmg` from GitHub Actions
3. Open normally (no `xattr -cr` needed!)
4. Verify: Right-click → Get Info → should show "Developer ID Application: Your Name"

## Benefits of Proper Signing

✅ **No "damaged" error** - Users can open normally
✅ **No `xattr -cr` needed** - Double-click works
✅ **Professional appearance** - Shows your name/organization
✅ **User trust** - Apple-verified developer
✅ **Automatic updates** - Can use Tauri updater seamlessly

## Costs

- **Apple Developer Program**: $99/year
- **Time**: ~2-4 hours initial setup
- **Maintenance**: Certificate renewal yearly

## Alternatives

If cost is a concern:

1. **Keep unsigned** - Current approach with `xattr -cr` workaround
2. **Document well** - Clear instructions (already done)
3. **Homebrew Cask** - If popular, users can install via `brew install`
4. **Build from source** - Power users can build locally

## Common Issues

### "Unable to find identity"
- Certificate not imported correctly
- Use `security find-identity -v -p codesigning` to verify

### "Notarization failed"
- Check Apple ID/password are correct
- App-specific password must be used (not main password)
- Ensure hardened runtime is enabled

### "Certificate has expired"
- Certificates expire after 1 year
- Renew in Apple Developer Portal
- Update GitHub Secrets with new certificate

## Resources

- [Apple Developer Portal](https://developer.apple.com/)
- [Tauri Signing Guide](https://tauri.app/v1/guides/distribution/sign-macos)
- [Notarization Documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [GitHub Actions Code Signing](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect)

---

**Note:** This is entirely optional. Many successful open-source macOS apps distribute unsigned builds with clear documentation about the `xattr -cr` workaround.
