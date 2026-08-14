use anyhow::{anyhow, Result};
use base64::Engine;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "audio-to-text")]
#[command(about = "Convert MP3 audio to text using Ollama + Whisper", long_about = None)]
struct Args {
    /// Input MP3 file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output text file path (optional, defaults to input_name.txt)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Ollama API endpoint
    #[arg(short, long, default_value = "http://localhost:11434")]
    ollama: String,

    /// Model to use for transcription
    #[arg(short, long, default_value = "whisper")]
    model: String,
}

#[derive(Serialize, Debug)]
struct TranscriptionRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct TranscriptionResponse {
    response: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Audio to text converter");
    info!("Input file: {:?}", args.input);
    info!("Ollama endpoint: {}", args.ollama);
    info!("Model: {}", args.model);

    // Validate input file exists
    if !args.input.exists() {
        return Err(anyhow!("Input file not found: {:?}", args.input));
    }

    // Determine output path
    let output_path = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("txt");
        path
    });

    info!("Output file: {:?}", output_path);

    // Read audio file
    info!("Reading audio file...");
    let audio_data = fs::read(&args.input)?;
    info!("Audio file size: {} bytes", audio_data.len());

    // Encode audio to base64
    let encoded_audio = base64::engine::general_purpose::STANDARD.encode(&audio_data);

    // Create transcription request
    let request = TranscriptionRequest {
        model: args.model.clone(),
        prompt: encoded_audio,
        stream: false,
    };

    // Send request to Ollama
    info!("Sending request to Ollama...");
    let client = Client::new();
    let response = client
        .post(&format!("{}/api/generate", args.ollama))
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Ollama returned error: {} {}",
            response.status(),
            response.text().await?
        ));
    }

    // Parse response
    let transcription: TranscriptionResponse = response.json().await?;
    info!("Transcription received, saving to file...");

    // Save to file
    fs::write(&output_path, &transcription.response)?;
    info!("Successfully saved transcription to: {:?}", output_path);

    println!("\n✓ Transcription completed!");
    println!("  Input:  {:?}", args.input);
    println!("  Output: {:?}", output_path);

    Ok(())
}
