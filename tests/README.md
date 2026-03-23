# KeVoiceInput Tests

This directory contains tests and testing utilities for KeVoiceInput.

## Directory Structure

```
tests/
├── README.md           # This file
├── integration/        # Integration tests (Rust)
├── fixtures/           # Test data
│   ├── audio/         # Test audio files
│   └── models/        # Mock/test model files
└── scripts/           # Test scripts
```

## Running Tests

### Integration Tests (Rust)

Currently, the project uses manual testing. Automated integration tests are planned for future development.

To run Rust unit tests:

```bash
cd src-tauri
cargo test
```

### Manual Testing Scripts

The `scripts/` directory contains various test scripts for specific functionality:

#### sherpa-onnx API Tests

Test sherpa-onnx library integration:

```bash
cd tests/scripts
./test_sherpa_api.sh
```

#### Local SeACo Paraformer Test

Test SeACo Paraformer model with hotword support:

```bash
cd tests/scripts
./test_local_sherpa_seaco.sh
```

#### API Model Tests

Test DashScope Qwen3-ASR-Flash model:

```bash
cd tests/scripts
./test_qwen3_asr_flash.sh
```

#### Crash Log Testing

Run tests with crash log capture:

```bash
cd tests/scripts
./run_test_with_crash_log.sh
```

#### Binary Download Tests

Test sherpa-onnx binary download and execution:

```bash
cd tests/scripts
./test-with-download-binaries.sh
```

## Testing Focus Areas

### Audio Recording
- Microphone input capture
- VAD (Voice Activity Detection) accuracy
- Audio buffer management
- Device switching

### Model Loading
- Model validation
- Engine initialization
- Model switching
- Error handling for corrupted models

### Transcription Accuracy
- Engine-specific tests (Whisper, Paraformer, Transducer, FireRedAsr)
- Hotword recognition (SeACo Paraformer)
- Multi-language support
- Real-time vs. offline processing

### Text Processing Pipeline
- Custom word correction
- Filler word removal
- ITN (Inverse Text Normalization)
- Auto-punctuation
- Hotword rules
- LLM post-processing

### Settings Persistence
- Settings save/load
- Default values
- Migration between versions

### UI/UX
- Keyboard shortcuts
- Overlay behavior
- Model download progress
- Error messages

## Creating New Tests

### Integration Tests (Rust)

Create test files in `integration/`:

```rust
// integration/audio_recording.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_recording() {
        // Test implementation
    }
}
```

### Test Fixtures

Add test data to `fixtures/`:

- **Audio files**: `fixtures/audio/` (e.g., `test_recording.wav`)
- **Mock models**: `fixtures/models/` (small test models)

### Test Scripts

Add shell scripts to `scripts/`:

```bash
#!/bin/bash
# Test script template

set -e

echo "Running test..."

# Test implementation

echo "✅ Test passed"
```

Make scripts executable:

```bash
chmod +x tests/scripts/your_test.sh
```

## Test Guidelines

1. **Keep tests focused**: One test per feature/bug
2. **Use meaningful names**: Describe what is being tested
3. **Clean up resources**: Remove temporary files after tests
4. **Document requirements**: Note dependencies (models, API keys, etc.)
5. **Provide examples**: Show expected input/output

## CI/CD

Automated testing via GitHub Actions is planned. See `.github/workflows/` for CI configuration.

## Contributing

When adding new features:

1. Write tests for the feature
2. Ensure existing tests still pass
3. Update this README if adding new test categories

See [CONTRIBUTING.md](../CONTRIBUTING.md) for general contribution guidelines.
