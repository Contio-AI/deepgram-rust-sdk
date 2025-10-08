//! Deepgram Flux Streaming Example
//!
//! This example demonstrates how to use Deepgram's Flux model for conversational
//! speech recognition with real-time turn detection and confidence scoring.
//!
//! Flux is specifically designed for voice agents and provides:
//! - Model-integrated turn detection (no separate VAD needed)
//! - Turn events: StartOfTurn, EagerEndOfTurn, TurnResumed, EndOfTurn
//! - Configurable end-of-turn thresholds for optimal latency vs accuracy
//! - Word-level confidence scoring with color-coded output
//!
//! This example streams live audio from BBC World Service and converts it to
//! the format expected by Flux using ffmpeg.
//!
//! Prerequisites: ffmpeg must be installed and available in PATH
//! Run with: cargo run --example flux_stream

use std::env;
use std::process::Stdio;

use tokio::io::AsyncReadExt;
use tokio::process::Command;

use deepgram::{
    common::{
        options::{Encoding, Model, Options},
        stream_response::{StreamResponse, TurnEvent},
    },
    Deepgram, DeepgramError,
};

// URL for live BBC World Service audio stream
static STREAM_URL: &str = "http://stream.live.vc.bbcmedia.co.uk/bbc_world_service";
// Audio chunk size for streaming
static AUDIO_CHUNK_SIZE: usize = 1024;

// Terminal color codes for confidence visualization
struct Colors;
impl Colors {
    const GREEN: &'static str = "\x1b[92m"; // High confidence (0.90-1.00)
    const YELLOW: &'static str = "\x1b[93m"; // Good confidence (0.80-0.90)
    const ORANGE: &'static str = "\x1b[91m"; // Medium confidence (0.70-0.80)
    const RED: &'static str = "\x1b[31m"; // Low confidence (<=0.69)
    const BLUE: &'static str = "\x1b[94m"; // For turn events
    const CYAN: &'static str = "\x1b[96m"; // For metadata
    const RESET: &'static str = "\x1b[0m"; // Reset to default
}

/// Get the appropriate color code based on confidence score
fn get_confidence_color(confidence: f64) -> &'static str {
    if confidence >= 0.90 {
        Colors::GREEN
    } else if confidence >= 0.80 {
        Colors::YELLOW
    } else if confidence >= 0.70 {
        Colors::ORANGE
    } else {
        Colors::RED
    }
}

/// Format turn event with appropriate emoji and description
fn format_turn_event(event: &TurnEvent) -> (&'static str, &'static str) {
    match event {
        TurnEvent::Update => ("🔄", "Ongoing transcription"),
        TurnEvent::StartOfTurn => ("🎤", "User started speaking"),
        TurnEvent::EagerEndOfTurn => ("⚡", "Likely end of turn (fast)"),
        TurnEvent::TurnResumed => ("🔄", "User continued speaking"),
        TurnEvent::EndOfTurn => ("✅", "Turn completed"),
    }
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    // Get API key from environment
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    // Create Deepgram client
    let dg_client = Deepgram::new(&deepgram_api_key)?;

    println!(
        "{}🚀 Deepgram Flux Streaming Example{}",
        Colors::CYAN,
        Colors::RESET
    );
    println!(
        "{}📡 Connecting to Flux model for conversational speech recognition...{}",
        Colors::CYAN,
        Colors::RESET
    );

    // Configure options for Flux model
    // Note: Flux v2 endpoint supports fewer parameters than v1
    let options = Options::builder()
        .model(Model::FluxGeneralEn) // Use Flux model - this determines the v2 endpoint
        .build();

    // Start ffmpeg process to convert BBC stream to linear16 PCM
    println!(
        "{}🌐 Starting to stream and convert audio from: {}{}",
        Colors::CYAN,
        STREAM_URL,
        Colors::RESET
    );

    let mut ffmpeg_process = Command::new("ffmpeg")
        .args(&[
            "-i", STREAM_URL, // Input: BBC World Service stream
            "-f", "s16le", // Output format: 16-bit little-endian PCM (linear16)
            "-ar", "16000", // Sample rate: 16kHz
            "-ac", "1", // Channels: mono
            "-", // Output to stdout
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| DeepgramError::InternalClientError(e.into()))?;

    let stdout = ffmpeg_process.stdout.take().ok_or_else(|| {
        DeepgramError::InternalClientError(anyhow::anyhow!("Failed to get ffmpeg stdout"))
    })?;

    println!(
        "{}✅ Audio conversion started (BBC → linear16 PCM){}",
        Colors::GREEN,
        Colors::RESET
    );

    // Create streaming connection with Flux-specific parameters
    // Note: Flux v2 endpoint doesn't support keep_alive messages
    let mut handle = dg_client
        .transcription()
        .stream_request_with_options(options)
        .encoding(Encoding::Linear16) // 16-bit PCM encoding
        .sample_rate(16000) // 16kHz sample rate (optimal for Flux)
        // Flux-specific parameters for turn detection
        .eot_threshold(0.8) // End-of-turn confidence threshold (0.5-0.9)
        .eager_eot_threshold(0.6) // Eager end-of-turn threshold (0.3-0.9)
        .eot_timeout_ms(5000) // Force timeout after 5 seconds
        .handle()
        .await?;

    println!(
        "{}✅ Connected to Deepgram Flux!{}",
        Colors::GREEN,
        Colors::RESET
    );
    println!(
        "{}📋 Request ID: {}{}",
        Colors::CYAN,
        handle.request_id(),
        Colors::RESET
    );
    println!(
        "{}🎯 Flux Parameters: eot_threshold=0.8, eager_eot_threshold=0.6, eot_timeout=5000ms{}",
        Colors::CYAN,
        Colors::RESET
    );
    println!();
    println!("{}📝 Legend:{}", Colors::CYAN, Colors::RESET);
    println!(
        "  {}● High confidence (0.90+){}",
        Colors::GREEN,
        Colors::RESET
    );
    println!(
        "  {}● Good confidence (0.80-0.89){}",
        Colors::YELLOW,
        Colors::RESET
    );
    println!(
        "  {}● Medium confidence (0.70-0.79){}",
        Colors::ORANGE,
        Colors::RESET
    );
    println!("  {}● Low confidence (<0.70){}", Colors::RED, Colors::RESET);
    println!();

    // Create async reader for ffmpeg output
    let mut reader = tokio::io::BufReader::new(stdout);
    let mut buffer = vec![0u8; AUDIO_CHUNK_SIZE];

    // Process streaming: send audio data and receive responses
    loop {
        tokio::select! {
            // Read and send audio data to Deepgram
            read_result = reader.read(&mut buffer) => {
                match read_result {
                    Ok(0) => {
                        // EOF - finalize the stream
                        println!("{}📡 Audio stream ended, finalizing...{}", Colors::CYAN, Colors::RESET);
                        if let Err(e) = handle.finalize().await {
                            eprintln!("Error finalizing stream: {}", e);
                        }
                        break;
                    }
                    Ok(n) => {
                        // Send audio chunk to Deepgram
                        if let Err(e) = handle.send_data(buffer[..n].to_vec()).await {
                            eprintln!("Error sending audio data: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading audio data: {}", e);
                        break;
                    }
                }
            }

            // Handle Deepgram responses
            result = handle.receive() => {
                match result {
                    Some(Ok(response)) => {
                        match response {
            // Handle Flux TurnInfo messages
            StreamResponse::TurnInfoResponse {
                event,
                turn_index,
                transcript,
                words,
                end_of_turn_confidence,
                audio_window_start,
                audio_window_end,
                ..
            } => {
                let (emoji, description) = format_turn_event(&event);

                // Display turn event information
                println!(
                    "{}{}[Turn {}] {} - {}{}",
                    Colors::BLUE,
                    emoji,
                    turn_index,
                    description,
                    Colors::RESET,
                    if matches!(event, TurnEvent::EndOfTurn | TurnEvent::EagerEndOfTurn) {
                        format!(" (confidence: {:.2})", end_of_turn_confidence)
                    } else {
                        String::new()
                    }
                );

                // Display transcript if available
                if !transcript.is_empty() {
                    println!("  {}💬 \"{}\"{}", Colors::CYAN, transcript, Colors::RESET);

                    // Display audio window timing
                    println!(
                        "  {}⏱️  Audio window: {:.2}s - {:.2}s{}",
                        Colors::CYAN,
                        audio_window_start,
                        audio_window_end,
                        Colors::RESET
                    );
                }

                // Display word-level confidence with color coding
                if let Some(words) = words {
                    if !words.is_empty() {
                        let colored_words: Vec<String> = words
                            .iter()
                            .map(|word| {
                                let color = get_confidence_color(word.confidence);
                                format!(
                                    "{}{}({:.2}){}",
                                    color,
                                    word.word,
                                    word.confidence,
                                    Colors::RESET
                                )
                            })
                            .collect();

                        println!(
                            "  {}📊 Words: {}{}",
                            Colors::CYAN,
                            colored_words.join(" | "),
                            Colors::RESET
                        );
                    }
                }

                println!(); // Add spacing between turn events
            }

            // Handle traditional transcript responses (for non-Flux models)
            StreamResponse::TranscriptResponse {
                channel,
                is_final,
                speech_final,
                ..
            } => {
                if let Some(alternative) = channel.alternatives.first() {
                    let status = if is_final { "FINAL" } else { "interim" };
                    let speech_status = if speech_final { " [SPEECH_FINAL]" } else { "" };

                    println!(
                        "{}[{}{}] {}{}",
                        if is_final {
                            Colors::GREEN
                        } else {
                            Colors::YELLOW
                        },
                        status,
                        speech_status,
                        alternative.transcript,
                        Colors::RESET
                    );
                }
            }

            // Handle other message types
            StreamResponse::SpeechStartedResponse { .. } => {
                println!("{}🎙️  Speech detected{}", Colors::BLUE, Colors::RESET);
            }

            StreamResponse::UtteranceEndResponse { .. } => {
                println!("{}🔇 Utterance ended{}", Colors::BLUE, Colors::RESET);
            }

            StreamResponse::TerminalResponse { duration, .. } => {
                println!(
                    "{}🏁 Stream completed - Duration: {:.2}s{}",
                    Colors::GREEN,
                    duration,
                    Colors::RESET
                );
            }

            StreamResponse::ConnectedResponse { request_id, .. } => {
                // Only show the first connected message to avoid spam
                // (Flux sends many connected messages)
                static FIRST_CONNECTED: std::sync::Once = std::sync::Once::new();
                FIRST_CONNECTED.call_once(|| {
                    println!(
                        "{}🔗 Connected to Flux - Request ID: {}{}",
                        Colors::GREEN,
                        request_id,
                        Colors::RESET
                    );
                });
            }

            StreamResponse::ErrorResponse {
                code, description, ..
            } => {
                println!(
                    "{}❌ Error: {} - {}{}",
                    Colors::RED,
                    code,
                    description,
                    Colors::RESET
                );
            }

                            // Handle any other message types that might be added in the future
                            _ => {
                                // Silently ignore unknown message types
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error receiving response: {}", e);
                        break;
                    }
                    None => {
                        println!("{}🔚 Connection closed{}", Colors::CYAN, Colors::RESET);
                        break;
                    }
                }
            }
        }
    }

    // Clean up
    let _ = ffmpeg_process.kill().await;

    println!(
        "{}✨ Flux streaming example completed!{}",
        Colors::GREEN,
        Colors::RESET
    );
    Ok(())
}
