# Contributing to KeVoiceInput

Thank you for your interest in contributing to KeVoiceInput! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Commit Convention](#commit-convention)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/KeVoiceInput.git
   cd KeVoiceInput
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/original-owner/KeVoiceInput.git
   ```
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- **Node.js** 18+ and **Bun** (or npm)
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Tauri CLI** (installed via project dependencies)
- **System Dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
  - Windows: Microsoft Visual Studio C++ Build Tools, WebView2 Runtime

### Installation

**macOS/Linux**:
```bash
# Install frontend dependencies
bun install  # or npm install

# Install Tauri CLI
cargo install tauri-cli

# Start development server
bun run tauri:dev
```

**Windows**:
```powershell
# Install frontend dependencies
bun install  # or npm install

# Install Tauri CLI (if needed)
cargo install tauri-cli

# Set sherpa-onnx library path
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"

# Start development server
bun run tauri:dev
```

For detailed build instructions, see [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md).

### Platform-Specific Setup

#### macOS Development

No additional setup required beyond standard prerequisites.

#### Windows Development

> **Note**: Windows support is currently under active development. See [docs/WINDOWS_PORT.md](docs/WINDOWS_PORT.md) for adaptation progress.

**Additional Requirements**:
1. Visual Studio Build Tools 2022 with:
   - Desktop development with C++
   - MSVC v143 or later
   - Windows 10 SDK
2. WebView2 Runtime
3. sherpa-onnx compiled for Windows (see [docs/WINDOWS_QUICKSTART.md](docs/WINDOWS_QUICKSTART.md))

**Environment Variables**:
```powershell
# Required: sherpa-onnx library path
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"

# Optional: Rust log level
$env:RUST_LOG = "debug"
```

**Using Build Scripts**:
```powershell
# Development mode
.\scripts\build-windows.ps1 -Dev

# Production build
.\scripts\build-windows.ps1
```

#### Linux Development

Linux support is planned. Contributions welcome!

## Project Structure

```
KeVoiceInput/
├── src/                    # Frontend React/TypeScript code
│   ├── components/        # React components
│   ├── stores/           # Zustand state management
│   ├── i18n/             # Internationalization
│   └── overlay/          # Recording overlay
├── src-tauri/             # Backend Rust code
│   ├── src/
│   │   ├── managers/     # Business logic managers
│   │   ├── commands/     # Tauri commands (IPC)
│   │   ├── audio_toolkit/ # Audio processing
│   │   └── main.rs       # Application entry
│   └── Cargo.toml
├── vendor/                # Vendored dependencies
│   └── sherpa-rs/        # Modified sherpa-rs
├── scripts/              # Build and utility scripts
├── docs/                 # Documentation
└── tests/                # Test files
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture information.

## Coding Standards

### Frontend (TypeScript/React)

- **Linting**: Run `bun run lint` before committing
- **Formatting**: Use Prettier (run `bun run format:frontend`)
- **Component Structure**:
  - Use functional components with hooks
  - Keep components small and focused
  - Extract reusable logic into custom hooks
- **State Management**:
  - Use Zustand for global state
  - Persist settings via Tauri commands
- **Naming Conventions**:
  - Components: PascalCase (`AudioRecorder.tsx`)
  - Files: kebab-case for utilities, PascalCase for components
  - Hooks: camelCase with `use` prefix (`useSettings.ts`)

### Backend (Rust)

- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Run `cargo clippy` to catch common issues
- **Error Handling**:
  - Use `Result<T, String>` for Tauri commands
  - Provide descriptive error messages
  - Log errors with `log::error!` for debugging
- **Naming Conventions**:
  - Files: snake_case (`audio_recording.rs`)
  - Modules: snake_case
  - Types: PascalCase (`AudioRecordingManager`)
  - Functions: snake_case (`start_recording`)
- **Architecture**:
  - Follow the Manager pattern (see [CLAUDE.md](CLAUDE.md))
  - Keep business logic in managers, not commands
  - Use `Arc<Mutex<Manager>>` for shared state

### Code Quality

- Write clear, self-documenting code
- Add comments for complex logic
- Avoid over-engineering (see CLAUDE.md guidelines)
- Don't add features beyond the scope of your PR
- Keep changes focused and minimal

### Cross-Platform Considerations

When contributing platform-specific code:

1. **Use Conditional Compilation**:
   ```rust
   #[cfg(target_os = "macos")]
   fn platform_specific_function() {
       // macOS implementation
   }

   #[cfg(target_os = "windows")]
   fn platform_specific_function() {
       // Windows implementation
   }
   ```

2. **Path Handling**:
   - Always use `PathBuf` and `Path` from `std::path`
   - Never hardcode path separators (`/` or `\`)
   - Use the `dirs` crate for platform-specific directories

3. **Dynamic Libraries**:
   - macOS: `.dylib`
   - Windows: `.dll`
   - Linux: `.so`
   - Handle library loading per platform

4. **Test on Target Platforms**:
   - If changing platform-specific code, test on that platform
   - Consider CI/CD implications for multi-platform builds
   - Document platform-specific behavior in comments

## Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, build, etc.)

**Examples**:
```
feat(audio): add support for custom VAD thresholds
fix(transcription): resolve Whisper model loading race condition
docs: update sherpa-onnx integration guide
chore: bump version to 0.0.2
```

## Pull Request Process

1. **Update your branch** with latest upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Test your changes**:
   ```bash
   # Run linters
   bun run lint
   cargo clippy

   # Build the project
   bun run build
   bun run tauri:build

   # Test manually in dev mode
   bun run tauri:dev
   ```

3. **Create a Pull Request**:
   - Use a clear, descriptive title
   - Reference related issues (`Fixes #123`)
   - Describe what changed and why
   - Include screenshots for UI changes
   - List breaking changes (if any)

4. **PR Review**:
   - Address reviewer feedback
   - Keep the PR focused (one feature/fix per PR)
   - Squash commits if requested

5. **Merging**:
   - PRs require approval from maintainers
   - CI checks must pass
   - Commits may be squashed on merge

## Testing

### Manual Testing

1. Test the affected functionality in dev mode
2. Verify on all supported platforms (if applicable)
3. Check for console errors/warnings
4. Test edge cases and error conditions

### Automated Testing

Currently, the project uses manual testing. Automated tests are welcome contributions!

**Testing Focus Areas**:
- Audio recording and playback
- Model loading and switching
- Transcription accuracy
- Text processing pipeline
- Settings persistence
- Keyboard shortcuts
- Error handling

## Documentation

### When to Update Documentation

- **New features**: Update relevant docs (README, API.md, etc.)
- **API changes**: Update [docs/API.md](docs/API.md)
- **Architecture changes**: Update [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **Build process changes**: Update [docs/BUILD_GUIDE.md](docs/BUILD_GUIDE.md)
- **Breaking changes**: Update [CHANGELOG.md](CHANGELOG.md)

### Documentation Style

- Write in clear, concise English
- Use code blocks for examples
- Include screenshots for UI features
- Update both English and Chinese docs (if applicable)

## Questions?

- Check [CLAUDE.md](CLAUDE.md) for codebase guidance
- Read [docs/](docs/) for technical documentation
- Open an issue for questions or discussions

Thank you for contributing to KeVoiceInput! 🎤
