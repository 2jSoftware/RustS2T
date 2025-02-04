# Rust Speech-to-Text (STT) Application

A real-time speech recognition application built in Rust that uses the Vosk speech recognition engine and provides a web interface for interaction.

![image](https://github.com/user-attachments/assets/6410d3c9-beae-422f-96e2-82323ed2127f)


## Features

- Real-time speech-to-text transcription
- Web-based user interface with WebSocket updates
- Support for multiple audio input devices
- Copy transcript functionality
- Visual recording indicator
- Offline speech recognition (no internet required)

## Quick Start

1. Install prerequisites:
   - Rust (2021 edition)
   - Audio input device (microphone)

2. Clone and build:
```bash
git clone <repository-url>
cd rust_stt
cargo build --release
```

3. Download Vosk model:
   - Get the small English model from [Vosk Models](https://alphacephei.com/vosk/models)
   - Extract to `model/vosk-model-small-en-us-0.15/`

4. Run the application:
```bash
cargo run --release
```

5. Open `http://localhost:3030` in your browser

## Technical Details

### Architecture
- **Frontend**: HTML/JavaScript web interface
- **Backend**: Rust server using Warp and WebSocket
- **Audio Processing**: 16kHz sampling, mono conversion, normalization

### Dependencies
- `vosk` (0.2.0) - Speech recognition engine
- `cpal` (0.15.2) - Audio input handling
- `tokio` (1.0) - Async runtime
- `warp` (0.3) - Web server

### Model Performance
- Accuracy: 10.38 (tedlium test), 9.85 (librispeech test-clean)
- Speed: 0.11xRT (desktop)
- Latency: 0.15s

## Model Information

The included Vosk model (vosk-model-small-en-us-0.15) has the following characteristics:
- Accuracy: 10.38 (tedlium test), 9.85 (librispeech test-clean)
- Speed: 0.11xRT (desktop)
- Latency: 0.15s (right context)
- Copyright 2020 Alpha Cephei Inc

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the MIT License
```
