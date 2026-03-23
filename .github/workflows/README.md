# GitHub Actions Workflows

This directory contains GitHub Actions workflows for automated CI/CD.

## Workflows

### 1. CI Workflow (`ci.yml`)

**Triggers**:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**Jobs**:
- **Lint and Format Check**: ESLint, Prettier, Rustfmt, Clippy
- **Build Test**: Multi-platform build verification (Ubuntu, macOS, Windows)
- **Version Check**: Ensures version consistency across all config files

**Usage**:
```bash
# Manually trigger (if enabled)
gh workflow run ci.yml
```

### 2. Release Workflow (`release.yml`)

**Triggers**:
- Push tags matching `v*` pattern (e.g., `v0.0.2`)

**Jobs**:
1. **create-release**: Creates GitHub Release with CHANGELOG notes
2. **build-tauri**: Multi-platform builds (macOS, Linux, Windows)
3. **update-latest-json**: Generates and commits `latest.json` for auto-updater

**Usage**:
```bash
# Create and push a version tag
./scripts/sync-version.sh 0.0.2
git add -A
git commit -m "chore: bump version to 0.0.2"
git tag -a v0.0.2 -m "Release 0.0.2"
git push origin main --tags

# Workflow automatically triggers
# Monitor at: https://github.com/yourusername/KeVoiceInput/actions
```

**Artifacts**:
- macOS: `.dmg`, `.app.tar.gz`, `.app.tar.gz.sig`
- Linux: `.AppImage`, `.deb`, `.AppImage.tar.gz`, `.AppImage.tar.gz.sig`
- Windows: `.msi`, `.msi.zip`, `.msi.zip.sig`
- `latest.json` for Tauri auto-updater

## Required Secrets

Configure these in GitHub Settings → Secrets and variables → Actions:

### Required for Release

- `TAURI_SIGNING_PRIVATE_KEY`: Content of `.tauri-updater.key` file
- `TAURI_KEY_PASSWORD`: Password for the signing key (if set)

### Optional for macOS Code Signing

- `APPLE_CERTIFICATE`: Base64-encoded signing certificate
- `APPLE_CERTIFICATE_PASSWORD`: Certificate password
- `APPLE_ID`: Apple ID email
- `APPLE_PASSWORD`: App-specific password
- `APPLE_TEAM_ID`: Apple Developer Team ID

## Setup Instructions

### 1. Add Tauri Signing Key

```bash
# Display your signing key
cat .tauri-updater.key

# Copy the entire content
# Go to GitHub → Settings → Secrets → New repository secret
# Name: TAURI_SIGNING_PRIVATE_KEY
# Value: (paste the key content)
```

### 2. (Optional) Configure macOS Code Signing

```bash
# Export certificate as base64
base64 -i YourCertificate.p12 | pbcopy

# Add to GitHub Secrets:
# - APPLE_CERTIFICATE (base64 string)
# - APPLE_CERTIFICATE_PASSWORD
# - APPLE_ID
# - APPLE_PASSWORD (app-specific, not your Apple ID password)
# - APPLE_TEAM_ID
```

Generate app-specific password:
1. Go to https://appleid.apple.com/account/manage
2. Sign in
3. Security → App-Specific Passwords → Generate

### 3. Enable Workflows

Workflows are enabled by default when these files are committed.

### 4. Test the Setup

Create a test release:

```bash
./scripts/sync-version.sh 0.0.2-test
git add -A
git commit -m "test: release workflow"
git tag -a v0.0.2-test -m "Test release"
git push origin main --tags
```

Check the Actions tab to see if it runs successfully.

## Troubleshooting

### Build Fails with "Version Mismatch"

Use the version sync script:
```bash
./scripts/sync-version.sh 0.0.2
```

### "TAURI_SIGNING_PRIVATE_KEY not found"

Ensure the secret is added in GitHub Settings and the name matches exactly.

### macOS Build Fails

Check:
- Rust target installed: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`
- Xcode Command Line Tools installed
- Code signing certificates valid

### Linux Build Fails

Check:
- All required system dependencies installed
- See `Install dependencies (Ubuntu)` step in workflow

### Windows Build Fails

Check:
- Visual Studio Build Tools installed
- Rust MSVC toolchain installed

### Release Assets Not Uploaded

Check:
- Signature files exist in `release-out/`
- Build artifacts created successfully
- GitHub token has write permissions

## Manual Release (Without CI)

If you prefer to build and release manually:

```bash
# Build all platforms locally
bun run tauri:build

# Create release artifacts
./scripts/release-artifacts.sh

# Generate latest.json
./scripts/generate-latest-json.sh 0.0.2

# Create GitHub Release manually
# Upload files from release-out/ directory
```

## Workflow Customization

### Change Platforms

Edit `release.yml` matrix:

```yaml
strategy:
  matrix:
    include:
      - platform: macos-latest
        target: aarch64-apple-darwin
        arch: aarch64
      # Add more platforms here
```

### Add Tests

Edit `ci.yml` to add test jobs:

```yaml
- name: Run tests
  working-directory: src-tauri
  run: cargo test
```

### Change Triggers

Edit workflow `on:` section:

```yaml
on:
  push:
    branches: [ main ]  # Only main branch
  schedule:
    - cron: '0 0 * * 0'  # Weekly
```

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Tauri GitHub Actions Guide](https://tauri.app/v1/guides/building/cross-platform)
- [actions/create-release](https://github.com/actions/create-release)
- [actions/upload-release-asset](https://github.com/actions/upload-release-asset)
