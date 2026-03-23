# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive project documentation and standardization

### Changed
- Improved .gitignore configuration
- Unified version management across all configuration files

### Removed
- Debug logging code from production builds

## [0.0.1] - 2026-03-04

### Added
- Multi-engine speech recognition support
  - Whisper (Small, Medium, Turbo, Large models)
  - Transducer (Zipformer bilingual, Conformer Chinese)
  - FireRedAsr (Large bilingual with dialect support)
  - Paraformer (KeSeaCo with hotword support)
- API engine integration
  - DashScope (Alibaba Cloud)
  - OpenAI Whisper API
  - Azure Speech
  - Custom API endpoints
- LLM post-processing support
  - OpenAI (GPT-4, GPT-3.5-turbo)
  - Anthropic (Claude)
  - Groq (Llama 3, Mixtral)
  - DashScope (Qwen)
  - Apple Intelligence (local LLM on macOS ARM64)
  - Custom LLM endpoints
- Advanced text processing pipeline
  - Custom word correction (Levenshtein + Soundex)
  - Filler word removal
  - Inverse Text Normalization (ITN)
  - Auto-punctuation
  - Hotword rules
- Voice Activity Detection (VAD) with Silero
- Audio recording with real-time transcription
- Transcription history with SQLite storage
- Audio playback of recordings
- 14-language internationalization (i18n)
- Automatic application updates
- Cross-platform support (macOS, Windows, Linux)
- Keyboard input simulation
- Customizable keyboard shortcuts
- Model download and management
- Model import from local files

### Technical
- Tauri 2.x backend with Rust
- React frontend with TypeScript
- Zustand state management
- Vendored sherpa-rs with SeACo Paraformer support
- sherpa-onnx integration for speech recognition
- whisper.cpp integration via transcribe-rs
- cpal for cross-platform audio I/O
- enigo for input simulation

[Unreleased]: https://github.com/yourusername/KeVoiceInput/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/yourusername/KeVoiceInput/releases/tag/v0.0.1
