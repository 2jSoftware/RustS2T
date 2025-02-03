use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use vosk::{Model, Recognizer};
use warp::ws::{Message, WebSocket};
use warp::Filter;
use tokio::sync::mpsc;
mod audio;
mod speech_recognition;
use speech_recognition::SpeechRecognizer; // Import from the declared module

#[derive(Debug, Serialize, Deserialize)]
struct TranscriptMessage {
    r#type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct ClientMessage {
    r#type: String,
    #[serde(default, alias = "isRecording")]
    is_recording: bool,
    #[serde(default)]
    device_id: String,
}

struct AudioState {
    is_recording: bool,
    recognizer: Recognizer,
    buffer: Vec<f32>,
    resampled_buffer: Vec<f32>,
    target_sample_rate: f64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize Vosk model
    println!("Loading Vosk model...");
    let model = Model::new("model/vosk-model-small-en-us-0.15/").ok_or(anyhow::anyhow!("Failed to create model"))?;
    let recognizer = Recognizer::new(&model, 16000.0).ok_or(anyhow::anyhow!("Failed to create recognizer"))?;

    let audio_state = Arc::new(Mutex::new(AudioState {
        is_recording: false,
        recognizer,
        buffer: Vec::with_capacity(32000), // 2 seconds at 16kHz
        resampled_buffer: Vec::with_capacity(32000),
        target_sample_rate: 16000.0,
    }));

    // Create a broadcast channel for sending transcripts to all connected clients
    let (tx, _) = broadcast::channel(16);
    let tx = Arc::new(tx);

    // Set up audio input
    println!("Setting up audio input...");
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(anyhow::anyhow!("No input device available"))?;

    println!("Using input device: {}", device.name()?);
    
    // Get supported configs
    let supported_configs = device.supported_input_configs()?;
    println!("Supported configurations:");
    for config in supported_configs {
        println!("  {:?}", config);
    }

    // Try to get a config close to what we want
    let config = device.default_input_config()?;
    println!("Selected config: {:?}", config);
    let sample_rate = config.sample_rate().0 as f64;
    println!("Input sample rate: {}", sample_rate);

    // Clone for audio processing
    let audio_state_clone = audio_state.clone();
    let tx_clone = tx.clone();

    // Start audio processing in a separate task
    tokio::task::spawn_blocking(move || {
        println!("Starting audio stream...");
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                process_audio(data, &audio_state_clone, &tx_clone, sample_rate);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None, // Remove the timeout
        ).unwrap();

        stream.play().unwrap();
        println!("Audio stream started");
        std::thread::sleep(Duration::from_secs(86400));
    });

    // Serve static files
    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let index = warp::path::end().and(warp::fs::file("static/index.html"));

    // WebSocket handler
    let ws_route = warp::path("ws").and(warp::ws()).map(move |ws: warp::ws::Ws| {
        let tx = tx.clone();
        let audio_state = audio_state.clone();
        ws.on_upgrade(move |socket| handle_websocket(socket, tx, audio_state))
    });

    // Combine routes
    let routes = static_files.or(index).or(ws_route);

    println!("Server running at http://localhost:3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

    // Create an asynchronous channel for recognized text
    let (tx_text, mut rx_text) = mpsc::channel::<String>(32);

    // Launch the speech recognizer asynchronously, passing the sender handle
    tokio::spawn(async move {
        let mut recognizer = SpeechRecognizer::new(tx_text);
        recognizer.run().await; // run the recognition event loop
    });

    // Main loop: listen for recognized text and output it in real time
    while let Some(text) = rx_text.recv().await {
        println!("Recognized: {}", text);
    }

    Ok(())
}

fn process_audio(
    data: &[f32],
    audio_state: &Arc<Mutex<AudioState>>,
    tx: &broadcast::Sender<String>,
    input_sample_rate: f64,
) {
    if let Ok(mut state) = audio_state.try_lock() {
        if !state.is_recording {
            return;
        }

        // Debug: Check if we're receiving audio data
        let max_amplitude = data.iter().fold(0.0f32, |max, &sample| max.max(sample.abs()));
        if max_amplitude > 0.01 {
            println!("Receiving audio: max amplitude = {}", max_amplitude);
        }

        // Convert stereo (2-channel) audio to mono by averaging the two channels
        if data.len() % 2 == 0 {
            let mono_samples: Vec<f32> = data
                .chunks_exact(2)
                .map(|chunk| (chunk[0] + chunk[1]) * 0.5)
                .collect();
            state.buffer.extend_from_slice(&mono_samples);
        } else {
            state.buffer.extend_from_slice(data);
        }

        // Process in chunks of about 1 second (16000 samples at 16kHz)
        let target_chunk_size = state.target_sample_rate as usize;
        let input_chunk_size = (target_chunk_size as f64 * input_sample_rate / state.target_sample_rate) as usize;

        while state.buffer.len() >= input_chunk_size {
            // Take a chunk from the buffer
            let chunk: Vec<f32> = state.buffer.drain(..input_chunk_size).collect();
            
            // Debug: Check chunk size and content
            println!("Processing chunk of {} samples", chunk.len());
            let chunk_max = chunk.iter().fold(0.0f32, |max, &sample| max.max(sample.abs()));
            println!("Chunk max amplitude: {}", chunk_max);
            
            // Resample to target sample rate (16kHz)
            state.resampled_buffer.clear();
            let step = input_sample_rate / state.target_sample_rate;
            let mut index = 0.0;
            while index < chunk.len() as f64 {
                let i = index.floor() as usize;
                if i < chunk.len() {
                    // Linear interpolation between samples
                    let fract = index - i as f64;
                    let current = chunk[i];
                    let next = if i + 1 < chunk.len() { chunk[i + 1] } else { current };
                    let sample = current + (next - current) * fract as f32;
                    state.resampled_buffer.push(sample);
                }
                index += step;
            }

            println!("Resampled to {} samples", state.resampled_buffer.len());

            // Normalize audio to prevent clipping
            let max_val = state.resampled_buffer.iter()
                .fold(0.0f32, |max, &sample| max.max(sample.abs()));
            if max_val > 0.0 {
                for sample in state.resampled_buffer.iter_mut() {
                    *sample /= max_val;
                }
            }

            // Convert to i16 samples
            let samples: Vec<i16> = state.resampled_buffer
                .iter()
                .map(|x| (x * i16::MAX as f32) as i16)
                .collect();

            println!("Processing {} i16 samples with Vosk", samples.len());

            // Process with Vosk
            match state.recognizer.accept_waveform(&samples) {
                vosk::DecodingState::Running => {
                    let partial = state.recognizer.partial_result().partial;
                    if !partial.is_empty() {
                        println!("Got partial result: {}", partial);
                        let msg = serde_json::to_string(&TranscriptMessage {
                            r#type: "partial".to_string(),
                            text: partial.to_string(),
                        }).unwrap();
                        let _ = tx.send(msg);
                    }
                }
                vosk::DecodingState::Finalized => {
                    let result = state.recognizer.result();
                    let text = match result {
                        vosk::CompleteResult::Single(single) => single.text.to_string(),
                        _ => String::new(),
                    };
                    if !text.is_empty() {
                        println!("Got final result: {}", text);
                        let msg = serde_json::to_string(&TranscriptMessage {
                            r#type: "final".to_string(),
                            text,
                        }).unwrap();
                        let _ = tx.send(msg);
                    }
                }
                vosk::DecodingState::Failed => {
                    eprintln!("Speech recognition failed");
                }
            }
        }
    }
}

async fn handle_websocket(
    ws: WebSocket,
    tx: Arc<broadcast::Sender<String>>,
    audio_state: Arc<Mutex<AudioState>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    // Forward messages from broadcast to websocket
    tokio::task::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming websocket messages
    while let Some(result) = ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(text) = msg.to_str() {
                println!("Received WebSocket message: {}", text);
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text) {
                    if client_msg.r#type == "recording_state" {
                        if let Ok(mut state) = audio_state.try_lock() {
                            state.is_recording = client_msg.is_recording;
                            println!("Recording state updated to: {}", state.is_recording);
                            if !state.is_recording {
                                state.buffer.clear();
                                state.resampled_buffer.clear();
                            }
                        }
                    }
                }
            }
        }
    }
}
