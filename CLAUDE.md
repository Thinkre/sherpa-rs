# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KeVoiceInput is a cross-platform desktop voice input application built with Tauri (Rust backend) + React (TypeScript frontend). It integrates multiple speech recognition engines (Whisper, Sherpa-ONNX, Paraformer, SeACo Paraformer) and provides real-time voice transcription with advanced text processing features.

## Development Commands

### Frontend Development
```bash
# Install dependencies
bun install          # or npm install

# Start frontend dev server only
bun run dev

# Build frontend
bun run build

# Lint and format
bun run lint         # Run ESLint
bun run lint:fix     # Auto-fix lint issues
bun run format       # Format all code (frontend + backend)
bun run format:frontend  # Format frontend only
bun run format:backend   # Format backend only (cargo fmt)
```

### Tauri Development
```bash
# Start Tauri development mode (frontend + backend)
bun run tauri:dev

# Build Tauri application
bun run tauri:build

# Run Tauri CLI commands
bun run tauri <command>
```

### Backend Development
```bash
# Format Rust code
cd src-tauri && cargo fmt

# Check Rust code
cd src-tauri && cargo check

# Clean build artifacts
cd src-tauri && cargo clean
```

## Critical Architecture Patterns

### Backend Architecture: Manager Pattern

The Rust backend uses a **Manager Pattern** where business logic is encapsulated in managers stored in Tauri's state:

- **ModelManager** (`managers/model.rs`): Model lifecycle (download, delete, import, list)
- **TranscriptionManager** (`managers/transcription.rs`): Core transcription logic, model loading/unloading, engine integration
- **AudioRecordingManager** (`managers/audio.rs`): Audio capture, VAD integration, recording lifecycle
- **HistoryManager** (`managers/history.rs`): SQLite storage for transcription history + audio file management
- **PunctuationManager** (`managers/punctuation.rs`): Auto-punctuation using sherpa-rs

**Key pattern**: Managers are accessed via `app_handle.state::<Arc<Mutex<Manager>>>()` in Tauri commands. Always acquire locks properly and handle errors.

### Frontend State Management: Zustand

Two main stores manage application state:
- **settingsStore** (`stores/settingsStore.ts`): App settings, audio devices, shortcuts
- **modelStore** (`stores/modelStore.ts`): Model list, active model, download progress

**Important**: Settings are persisted to disk via Tauri commands. Always sync state changes between Zustand store and backend.

### Transcription Engine Integration

The app supports multiple engines through a unified interface in `TranscriptionManager`:

1. **Whisper**: Via `transcribe-rs` crate (high accuracy, multi-language)
2. **Paraformer**: Via `sherpa-rs` (fast, Chinese-optimized)
3. **SeACo Paraformer**: Via `sherpa-rs` + `model_eb.onnx` (supports hotwords)
4. **Transducer**: Via `sherpa-rs` (streaming, low latency)

**Critical**: SeACo Paraformer requires `model_eb.onnx` file in addition to standard `model.onnx` + `tokens.txt`. Check for this file to distinguish it from standard Paraformer.

### Audio Pipeline Flow

```
User Shortcut → AudioRecordingManager → VAD → Audio Buffer
                                                    ↓
                                          TranscriptionManager
                                                    ↓
                                    [Text Processing Pipeline]
                                                    ↓
                          CustomWords → ITN → Punctuation → HotwordRules → LLM
                                                    ↓
                                          HistoryManager → Input Simulation
```

**Key components**:
- **VAD** (`audio_toolkit/vad/`): Silero VAD with smoothing to detect speech activity
- **Text Processing** (`audio_toolkit/text.rs`): Custom word correction (Levenshtein + Soundex), filler word removal, ITN (Inverse Text Normalization), hotword rules
- **Input Simulation** (`input.rs`): Uses `enigo` to type transcribed text

### sherpa-rs Integration

The project uses a **local modified version** of sherpa-rs in `vendor/sherpa-rs/` that adds `model_eb.onnx` support for SeACo Paraformer hotwords.

**Important environment variables**:
- `SHERPA_LIB_PATH`: Points to locally compiled sherpa-onnx libraries
- `DYLD_LIBRARY_PATH` (macOS): Runtime library search path

Dynamic libraries must be bundled into the app on macOS. See build scripts in `scripts/` for bundling logic.

### Multi-Entry Point Frontend

The Vite config defines two entry points:
1. **main**: Primary app UI (`index.html`)
2. **overlay**: Recording overlay UI (`src/overlay/index.html`)

Both are built separately and loaded by Tauri windows.

## File Locations

### Configuration & Data
- **Settings**: macOS `~/Library/Application Support/com.kevoiceinput.app/`
- **Models**: `<app_data>/models/`
- **History DB**: `<app_data>/history.db` (SQLite)
- **Audio Recordings**: `<app_data>/recordings/`
- **Logs**: Check Tauri logs plugin output

### Key Files
- **Backend Entry**: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- **Commands**: `src-tauri/src/commands/mod.rs` (exports all Tauri commands)
- **Frontend Entry**: `src/main.tsx`
- **Shortcuts**: `src-tauri/src/shortcut.rs` (registration), `src-tauri/src/actions.rs` (execution)
- **Settings Schema**: `src-tauri/src/settings.rs` (AppSettings struct)

## Development Guidelines

### When Modifying Tauri Commands

1. Update command definition in `src-tauri/src/commands/`
2. Re-export in `commands/mod.rs`
3. Add to `collect_commands!` macro in `src-tauri/src/lib.rs`
4. TypeScript bindings are auto-generated via `tauri-specta` (check `src/bindings.ts`)

### When Adding New Settings

1. Add field to `AppSettings` struct in `settings.rs`
2. Set default value in `get_default_settings()`
3. Create Tauri command for updating the setting
4. Update frontend store (`settingsStore.ts`)
5. Add UI component in `src/components/settings/`

### When Adding Speech Recognition Engines

1. Add engine type to `EngineType` enum
2. Implement loading logic in `TranscriptionManager::load_model()`
3. Add transcription logic in `TranscriptionManager::transcribe()`
4. Update model validation in `ModelManager`
5. Update frontend model selector

### Text Processing Pipeline

When modifying transcription output processing, follow this order in `TranscriptionManager::transcribe()`:
1. Raw transcription from engine
2. Filler word filtering (`filter_transcription_output`)
3. Custom word correction (`apply_custom_words`)
4. ITN normalization (`apply_itn`)
5. Auto-punctuation (via `PunctuationManager`)
6. Hotword rules (`apply_hot_rules`)
7. LLM post-processing (if enabled)

### Building for Production

**macOS**:
```bash
# Quick build (includes all bundling and signing)
bun run tauri:build

# The build process automatically:
# - Bundles sherpa-onnx dynamic libraries into .app
# - Adjusts library paths with install_name_tool
# - Creates DMG with auto-installer
# - Code signs (if developer cert available)
```

Build artifacts: `src-tauri/target/release/bundle/`

## Internationalization (i18n)

The app supports 14 languages with translation files in `src/i18n/locales/`. When adding UI text:
1. Add key to `en/translation.json`
2. Use `useTranslation()` hook in React components
3. Reference key with `t('your.key')`
4. Translations are managed manually in locale files

## Common Pitfalls

1. **Model Loading Errors**: Always check model file structure matches engine requirements (e.g., SeACo needs `model_eb.onnx`)

2. **Library Path Issues (macOS)**: If sherpa-onnx fails to load, verify:
   - Libraries are in `KeVoiceInput.app/Contents/Frameworks/`
   - `@rpath` is set correctly with `install_name_tool`
   - DYLD_LIBRARY_PATH is configured (dev mode)

3. **State Management**: Always sync Zustand stores with backend via Tauri commands. Don't assume store updates automatically persist.

4. **Audio Device Changes**: Handle device disconnection gracefully. AudioRecordingManager maintains device list, but UI must refresh on changes.

5. **VAD Tuning**: VAD parameters significantly affect recording quality. Adjust thresholds in `audio_toolkit/vad/` carefully.

6. **Tauri IPC**: Commands must be async and handle errors properly. Use `Result<T, String>` return type for better error messages.

7. **DMG Installer Script**: Use **English filenames** for `.command` scripts in DMG. Chinese characters in paths can cause terminal parsing issues on some Mac configurations. Current script name: `Install.command` (not `安装到应用程序.command`).

## Testing

There's a test binary for sherpa-rs integration:
```bash
cd src-tauri
cargo run --bin test_sherpa_api
```

This validates sherpa-onnx library loading and model initialization without running the full app.

## Important Notes

- This is a **local-first application** - all processing happens on-device except optional LLM post-processing
- The vendored `sherpa-rs` in `vendor/` is modified - don't replace with upstream without checking model_eb support
- macOS requires Accessibility permissions for keyboard input simulation
- Vite config includes debug logging (search for "agent log" comments) - can be removed in production
