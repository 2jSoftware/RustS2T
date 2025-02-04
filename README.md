# Rust Speech-to-Text (S2T) Application

A real-time speech recognition application built in Rust that uses the Vosk speech recognition engine and provides a web interface for interaction.

![image](https://github.com/user-attachments/assets/6410d3c9-beae-422f-96e2-82323ed2127f)


## Features

- Real-time speech-to-text transcription
- Web-based user interface with WebSocket updates
- Support for multiple audio input devices
- Copy transcript functionality
- Visual recording indicator
- Offline speech recognition (no internet required)

## Prerequisites

1. Install Rust (2021 edition)
2. Audio input device (microphone)
3. Vosk library files:
   - Download [vosk-win64-0.3.45.zip](https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-win64-0.3.45.zip)
   - Extract the ZIP file
   - Copy `libvosk.lib` and `vosk.dll` from the extracted folder to your project's root directory
   - These files are required for the application to build and run properly

## Quick Start

1. Clone and prepare the project:
```bash
git clone <repository-url>
cd RustS2T
```

2. Set up Vosk model:
   - Download the small English model from [Vosk Models](https://alphacephei.com/vosk/models)
   - Create a 'model' directory in the project root
   - Extract the model to `model/vosk-model-small-en-us-0.15/`

3. Build and run:
```bash
cargo build 
cargo run 
```

4. Open `http://localhost:3030` in your browser

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

The current used Vosk model (vosk-model-small-en-us-0.15) has the following characteristics:
- Accuracy: 10.38 (tedlium test), 9.85 (librispeech test-clean)
- Speed: 0.11xRT (desktop)
- Latency: 0.15s (right context)
- Copyright 2020 Alpha Cephei Inc

## Troubleshooting

If you encounter build errors related to `libvosk.lib`:
1. Ensure you've downloaded and extracted vosk-win64-0.3.45.zip
2. Verify that `libvosk.lib` and `vosk.dll` are present in your project's root directory
3. If issues persist, try adding the directory containing these files to your system's PATH environment variable

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

