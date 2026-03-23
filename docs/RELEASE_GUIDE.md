# KeVoiceInput Release Guide

This document describes the release process for KeVoiceInput.

## Prerequisites

- Write access to the GitHub repository
- Apple Developer account (for macOS code signing)
- Tauri signing key (stored in GitHub Secrets)
- Clean working directory

## Release Checklist

### 1. Pre-Release

- [ ] All tests pass
- [ ] No critical bugs in the issue tracker
- [ ] Documentation is up-to-date
- [ ] CHANGELOG.md updated with release notes
- [ ] All PRs merged to main branch

### 2. Version Management

Use the `scripts/sync-version.sh` script to update version across all files:

```bash
./scripts/sync-version.sh 0.0.2
```

This updates:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**Manual Verification**:
```bash
# Verify all versions match
grep version package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
```

### 3. Update CHANGELOG

Edit `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [0.0.2] - 2026-03-XX

### Added
- New feature X
- New feature Y

### Changed
- Improved Z

### Fixed
- Bug fix for W

[0.0.2]: https://github.com/yourusername/KeVoiceInput/compare/v0.0.1...v0.0.2
```

### 4. Commit and Tag

```bash
# Commit version bump
git add -A
git commit -m "chore: bump version to 0.0.2"

# Create annotated tag
git tag -a v0.0.2 -m "Release 0.0.2"

# Push changes and tags
git push origin main
git push origin v0.0.2
```

**Important**: The tag must follow the format `vX.Y.Z` (with `v` prefix) to trigger the release workflow.

### 5. Build Release Artifacts

#### Option A: GitHub Actions (Recommended)

Pushing a tag automatically triggers the release workflow (`.github/workflows/release.yml`).

Monitor the build progress:
```
https://github.com/yourusername/KeVoiceInput/actions
```

#### Option B: Manual Build

If automatic build fails or you need to build locally:

```bash
# Build for current platform
bun run tauri:build

# Find artifacts
ls src-tauri/target/release/bundle/
```

**macOS artifacts**:
- `macos/KeVoiceInput.app` - Application bundle
- `dmg/KeVoiceInput_X.Y.Z_aarch64.dmg` - DMG installer
- `dmg/KeVoiceInput_X.Y.Z_aarch64.app.tar.gz` - Tarball for auto-updater
- `dmg/KeVoiceInput_X.Y.Z_aarch64.app.tar.gz.sig` - Signature file

### 6. Generate Update Manifest

Run the script to generate `latest.json`:

```bash
./scripts/generate-latest-json.sh 0.0.2
```

This creates/updates `latest.json` with download URLs and signatures for all platforms.

**Verify `latest.json`**:
```bash
cat latest.json
# Check:
# - Version is correct
# - All platform URLs are valid
# - Signatures are present
# - Release notes are included
```

### 7. Create GitHub Release

#### Automated (via GitHub Actions)

The release workflow automatically:
1. Builds for all platforms
2. Creates GitHub Release
3. Uploads artifacts
4. Updates `latest.json`

#### Manual

If building manually:

1. Go to https://github.com/yourusername/KeVoiceInput/releases/new
2. Select the tag: `v0.0.2`
3. Title: `v0.0.2`
4. Description: Copy from CHANGELOG.md
5. Upload artifacts:
   - DMG (macOS)
   - AppImage (Linux)
   - MSI (Windows)
   - Tarball + signature files for auto-updater
6. Publish release

### 8. Update Auto-Updater Endpoint

If hosting `latest.json` separately from GitHub:

```bash
# Upload to your server
scp latest.json user@yourserver.com:/path/to/updates/

# Or commit to a separate branch
git checkout gh-pages
cp latest.json .
git add latest.json
git commit -m "Update to v0.0.2"
git push origin gh-pages
git checkout main
```

Update `tauri.conf.json` endpoints if needed:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://yourserver.com/updates/latest.json"
      ]
    }
  }
}
```

### 9. Test Auto-Update

1. Install previous version
2. Start the app
3. Check "About" or "Updates" menu
4. Verify update prompt appears
5. Test update installation

### 10. Announce

- [ ] Post to GitHub Discussions
- [ ] Update README badges if needed
- [ ] Social media announcements (if applicable)
- [ ] Email notifications (if applicable)

## Hotfix Releases

For critical bug fixes between regular releases:

1. Create hotfix branch:
   ```bash
   git checkout -b hotfix/0.0.3 v0.0.2
   ```

2. Apply fixes and commit

3. Update version:
   ```bash
   ./scripts/sync-version.sh 0.0.3
   ```

4. Update CHANGELOG (hotfix section)

5. Merge and release:
   ```bash
   git checkout main
   git merge hotfix/0.0.3
   git tag -a v0.0.3 -m "Hotfix 0.0.3"
   git push origin main --tags
   ```

## Rollback

If a release has critical issues:

1. **Do not delete the GitHub Release** - users may already be downloading it

2. Create a new patch version with fixes:
   ```bash
   ./scripts/sync-version.sh 0.0.4
   # Fix issues
   git commit -am "fix: critical issue from v0.0.3"
   git tag -a v0.0.4 -m "Fix critical issues"
   git push origin main --tags
   ```

3. Update `latest.json` to point to the fixed version

4. Add note to v0.0.3 release about issues and recommend upgrading

## Security

### Signing Keys

- **Never commit** `.tauri-updater.key` or private keys to the repository
- Store private key in GitHub Secrets: `TAURI_SIGNING_PRIVATE_KEY`
- Keep backup of keys in secure location (password manager)

### Code Signing (macOS)

Configure environment variables for code signing:

```bash
export APPLE_CERTIFICATE_BASE64=<base64-encoded-cert>
export APPLE_CERTIFICATE_PASSWORD=<cert-password>
export APPLE_ID=<your-apple-id>
export APPLE_PASSWORD=<app-specific-password>
export APPLE_TEAM_ID=<team-id>
```

Store these in GitHub Secrets for CI builds.

## Troubleshooting

### Build fails with "Version mismatch"

Ensure all version fields match:
```bash
./scripts/sync-version.sh <version>
git diff  # Verify changes
```

### Auto-update not working

1. Check `latest.json` is accessible
2. Verify signature matches the artifact
3. Check `tauri.conf.json` has correct pubkey
4. Test with `curl`:
   ```bash
   curl https://yourserver.com/updates/latest.json
   ```

### DMG not mounting on user's Mac

Ensure all dynamic libraries are bundled:
```bash
ls -la src-tauri/target/release/bundle/macos/KeVoiceInput.app/Contents/Frameworks/
# Should show all .dylib files
```

Re-run post-bundle script if needed:
```bash
./scripts/post-bundle.sh
```

## Release Schedule

Recommended release cadence:

- **Major releases** (1.0.0): Breaking changes, major features
- **Minor releases** (0.x.0): New features, monthly or bi-monthly
- **Patch releases** (0.0.x): Bug fixes, as needed
- **Hotfixes**: Critical issues only, within 24-48h

## Versioning Strategy

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: Backwards-compatible functionality
- **PATCH** version: Backwards-compatible bug fixes

For pre-1.0 versions (0.y.z):
- MINOR version may include breaking changes
- PATCH version for all other changes

## Useful Commands

```bash
# Check current version
grep version package.json | head -1

# List all tags
git tag -l

# View commits since last release
git log $(git describe --tags --abbrev=0)..HEAD --oneline

# Generate release notes from commits
git log $(git describe --tags --abbrev=0)..HEAD --pretty=format:"- %s" --reverse

# Clean build
cd src-tauri && cargo clean && cd ..
rm -rf dist node_modules/.vite
bun install
bun run tauri:build
```

## Resources

- [Tauri Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
