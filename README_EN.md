# KeVoiceInput

> English documentation is currently being prepared. For now, please refer to the Chinese README.

KeVoiceInput is a powerful desktop voice input application with multi-engine speech recognition support.

## Quick Links

- [Chinese README (完整中文文档)](README.md)
- [Build Guide](docs/BUILD_GUIDE.md)
- [API Documentation](docs/API.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Contributing Guide](CONTRIBUTING.md)

## Key Features

- **Multi-Engine Support**: Whisper, Paraformer, SeACo Paraformer, Transducer, FireRedAsr
- **API Integration**: DashScope, OpenAI Whisper API, Azure Speech, custom endpoints
- **LLM Post-Processing**: OpenAI, Anthropic (Claude), Groq, DashScope, Apple Intelligence
- **Local-First**: All processing can run offline for privacy
- **14 Languages**: Full internationalization support
- **Auto-Update**: Seamless updates via Tauri updater
- **Cross-Platform**: macOS, Windows, Linux

## Supported Models

### Local Engines

| Engine | Model | Language | Size | Accuracy | Speed | Features |
|--------|-------|----------|------|----------|-------|----------|
| Whisper | Small | Multi | 487MB | Good | Fast | General purpose |
| Whisper | Medium | Multi | 492MB | Excellent | Medium | Balanced |
| Whisper | Turbo | Multi | 1.6GB | Excellent | Medium | Recommended |
| Whisper | Large | Multi | 1.1GB | Best | Slow | Highest accuracy |
| Transducer | Zipformer | CN/EN | 320MB | Excellent | Fast | Hotword support |
| Transducer | Conformer | Chinese | 50MB | Excellent | Fast | CN hotwords |
| FireRedAsr | Large | CN/EN | 1.7GB | Best | Medium | Dialect support |
| Paraformer | KeSeaCo | Chinese | 50MB | Excellent | Fast | Hotword support |

See [docs/LOCAL_MODELS.md](docs/LOCAL_MODELS.md) for details.

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/yourusername/KeVoiceInput.git
cd KeVoiceInput

# Install dependencies
bun install

# Development mode
bun run tauri:dev

# Production build
bun run tauri:build
```

### From Release

Download installers from [Releases](https://github.com/yourusername/KeVoiceInput/releases).

## Quick Start

1. **Launch** the application
2. **Grant permissions** (accessibility on macOS, microphone access)
3. **Download a model** from the Models page
4. **Start recording** with the configured shortcut
5. **Speak** and see transcription appear

## Documentation

- [BUILD_GUIDE.md](docs/BUILD_GUIDE.md) - Detailed build instructions
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - System architecture
- [API.md](docs/API.md) - API documentation
- [SHERPA_ONNX.md](docs/SHERPA_ONNX.md) - sherpa-onnx integration
- [DEBUGGING.md](docs/DEBUGGING.md) - Troubleshooting guide
- [RELEASE_GUIDE.md](docs/RELEASE_GUIDE.md) - Release process

## sherpa-onnx Integration

This project uses a **vendored version** of sherpa-rs located in `vendor/sherpa-rs/`.

**Key Features**:
- SeACo Paraformer hotword support via `model_eb.onnx`
- Custom modifications for KeVoiceInput
- Bundled with compatible sherpa-onnx libraries

See [docs/SHERPA_ONNX.md](docs/SHERPA_ONNX.md) for integration details.

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgements

### Core Technologies

- [Tauri](https://tauri.app/) - Desktop application framework
- [Sherpa-ONNX](https://github.com/k2-fsa/sherpa-onnx) - Speech recognition
- [sherpa-rs](https://github.com/thewh1teagle/sherpa-rs) - Rust bindings
- [Whisper](https://github.com/openai/whisper) - OpenAI's speech model
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) - Whisper inference
- [transcribe-rs](https://github.com/thewh1teagle/transcribe-rs) - Transcription wrapper

### Speech Recognition Models

- [FunASR](https://github.com/alibaba-damo-academy/FunASR) - Alibaba DAMO Academy
  - Paraformer & SeACo Paraformer
- [K2 FSA](https://github.com/k2-fsa/) - k2-fsa community
  - Zipformer, Transducer, FireRedAsr

### Libraries

- [React](https://react.dev/) - UI framework
- [Tailwind CSS](https://tailwindcss.com/) - CSS framework
- [Vite](https://vitejs.dev/) - Build tool
- [Zustand](https://github.com/pmndrs/zustand) - State management
- [cpal](https://github.com/RustAudio/cpal) - Audio I/O
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [enigo](https://github.com/enigo-rs/enigo) - Input simulation

### Special Thanks

- k2-fsa community for excellent speech models
- Tauri team for the amazing framework
- All contributors and testers

### Inspiration

- [Handy](https://github.com/cjpais/Handy) - UI/UX inspiration
- [vibe](https://github.com/thewh1teagle/vibe) - Architecture inspiration

## Links

- [Issues](https://github.com/yourusername/KeVoiceInput/issues)
- [Discussions](https://github.com/yourusername/KeVoiceInput/discussions)

---

**Note**: Full English translation is in progress. For detailed information, please refer to the [Chinese README](README.md).
