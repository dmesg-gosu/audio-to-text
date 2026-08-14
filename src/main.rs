use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "audio-to-text")]
#[command(about = "Convert MP3 audio to text using OpenAI Whisper", long_about = None)]
struct Args {
    /// Input MP3 file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output text file path (optional, defaults to input_name.txt)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Whisper model size: tiny, base, small, medium, large
    #[arg(short, long, default_value = "base")]
    model: String,

    /// Output format: txt, vtt, srt, tsv, json, all
    #[arg(short = 'f', long, default_value = "txt")]
    format: String,

    /// Language code (e.g., en, ru, fr). Auto-detect if not specified
    #[arg(short, long)]
    language: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Audio to text converter (OpenAI Whisper)");
    info!("Input file: {:?}", args.input);
    info!("Model: {}", args.model);
    info!("Format: {}", args.format);

    // Validate input file exists
    if !args.input.exists() {
        return Err(anyhow!("Input file not found: {:?}", args.input));
    }

    // Determine output directory and filename
    let input_file = args.input.canonicalize()?;
    let output_dir = input_file
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine output directory"))?;

    let output_file_name = if let Some(output) = args.output {
        output
    } else {
        let mut path = input_file.file_stem().unwrap().to_os_string();
        path.push(".txt");
        output_dir.join(path)
    };

    info!("Output file: {:?}", output_file_name);
    info!("Starting transcription...");

    // Build whisper command
    let mut cmd = Command::new("whisper");
    cmd.arg(input_file.to_str().unwrap())
        .arg("--model")
        .arg(&args.model)
        .arg("--output_format")
        .arg(&args.format)
        .arg("--output_dir")
        .arg(output_dir.to_str().unwrap());

    // Add language if specified
    if let Some(lang) = args.language {
        cmd.arg("--language").arg(lang);
    }

    // Execute whisper
    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Whisper transcription failed:\n{}",
            stderr
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("Transcription output:\n{}", stdout);

    // Read the generated text file
    let mut txt_path = input_file.file_stem().unwrap().to_os_string();
    txt_path.push(".txt");
    let txt_file = output_dir.join(txt_path);

    if !txt_file.exists() {
        return Err(anyhow!(
            "Output file not created: {:?}",
            txt_file
        ));
    }

    // If custom output path specified, move the file
    if args.output.is_some() && txt_file != output_file_name {
        fs::rename(&txt_file, &output_file_name)?;
    }

    info!("Successfully saved transcription to: {:?}", output_file_name);

    println!("\n✓ Transcription completed!");
    println!("  Input:  {:?}", args.input);
    println!("  Output: {:?}", output_file_name);

    Ok(())
}
