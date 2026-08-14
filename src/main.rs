use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "audio-to-text")]
#[command(about = "Convert MP3 audio to text using OpenAI Whisper with live output", long_about = None)]
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

    /// Language code (e.g., en, ru, fr). Auto-detect if not specified
    #[arg(short, long)]
    language: Option<String>,

    /// Enable speaker diarization (requires pyannote.audio)
    #[arg(long)]
    diarize: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    println!("\n🎤 Audio to Text Converter (OpenAI Whisper)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Input file: {:?}", args.input);
    info!("Model: {}", args.model);
    if args.diarize {
        info!("Speaker diarization: enabled");
    }

    // Validate input file exists
    if !args.input.exists() {
        return Err(anyhow!("Input file not found: {:?}", args.input));
    }

    // Determine output directory and filename
    let input_file = args.input.canonicalize()?;
    let output_dir = input_file
        .parent()
        .ok_or_else(|| anyhow!("Cannot determine output directory"))?;

    let has_custom_output = args.output.is_some();
    let output_file_name = if let Some(output) = args.output.as_ref() {
        output.clone()
    } else {
        let mut path = input_file.file_stem().unwrap().to_os_string();
        path.push(".txt");
        output_dir.join(path)
    };

    info!("Output file: {:?}", output_file_name);
    println!("\n⏱️  Starting transcription...\n");

    // Build whisper command with json output for parsing
    let mut cmd = Command::new("whisper");
    cmd.arg(input_file.to_str().unwrap())
        .arg("--model")
        .arg(&args.model)
        .arg("--output_format")
        .arg("vtt")
        .arg("--output_dir")
        .arg(output_dir.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Add diarization if requested
    if args.diarize {
        cmd.arg("--diarize");
    }

    // Add language if specified
    if let Some(lang) = &args.language {
        cmd.arg("--language").arg(lang);
    }

    // Execute whisper with live output
    let mut child = cmd.spawn()?;

    // Capture stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.is_empty() && !line.starts_with("WEBVTT") {
                    println!("  {}", line);
                }
            }
        }
    }

    // Wait for completion
    let status = child.wait()?;

    if !status.success() {
        return Err(anyhow!("Whisper transcription failed"));
    }

    // Read the generated VTT file to calculate pauses
    let mut vtt_path = input_file.file_stem().unwrap().to_os_string();
    vtt_path.push(".vtt");
    let vtt_file = output_dir.join(vtt_path);

    if !vtt_file.exists() {
        return Err(anyhow!("VTT output file not created: {:?}", vtt_file));
    }

    // Parse VTT and calculate statistics
    println!("\n\n📊 Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let stats = parse_vtt_and_calculate_stats(&vtt_file)?;
    print_statistics(&stats);

    // Convert VTT to TXT if needed
    let txt_content = vtt_to_txt(&vtt_file)?;

    // If custom output path specified, use it; otherwise use default
    let final_output = if has_custom_output {
        output_file_name.clone()
    } else {
        let mut path = input_file.file_stem().unwrap().to_os_string();
        path.push(".txt");
        output_dir.join(path)
    };

    fs::write(&final_output, txt_content)?;

    println!("\n✓ Transcription completed!");
    println!("  Output: {:?}", final_output);
    println!();

    Ok(())
}

#[derive(Debug)]
struct VttStats {
    total_duration_seconds: f64,
    total_pause_seconds: f64,
    pause_count: usize,
    phrases: Vec<Phrase>,
}

#[derive(Debug)]
struct Phrase {
    start: f64,
    end: f64,
    text: String,
}

fn parse_vtt_and_calculate_stats(vtt_file: &PathBuf) -> Result<VttStats> {
    let content = fs::read_to_string(vtt_file)?;
    let mut phrases = Vec::new();
    let mut total_pause_seconds = 0.0;
    let mut pause_count = 0;

    for line in content.lines().skip(1) {
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() == 2 {
                let start = time_to_seconds(parts[0].trim());
                let end = time_to_seconds(parts[1].trim());
                phrases.push((start, end));
            }
        }
    }

    // Calculate pauses
    for i in 1..phrases.len() {
        let pause = phrases[i].0 - phrases[i - 1].1;
        if pause > 0.1 {
            total_pause_seconds += pause;
            pause_count += 1;
        }
    }

    let total_duration = if !phrases.is_empty() {
        phrases[phrases.len() - 1].1
    } else {
        0.0
    };

    Ok(VttStats {
        total_duration_seconds: total_duration,
        total_pause_seconds,
        pause_count,
        phrases: vec![],
    })
}

fn time_to_seconds(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() >= 3 {
        let hours = parts[0].parse::<f64>().unwrap_or(0.0);
        let minutes = parts[1].parse::<f64>().unwrap_or(0.0);
        let seconds_str = parts[2].replace(',', ".");
        let seconds = seconds_str.parse::<f64>().unwrap_or(0.0);
        hours * 3600.0 + minutes * 60.0 + seconds
    } else {
        0.0
    }
}

fn print_statistics(stats: &VttStats) {
    let hours = (stats.total_duration_seconds / 3600.0).floor() as u32;
    let minutes = ((stats.total_duration_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = (stats.total_duration_seconds % 60.0).floor() as u32;

    let pause_hours = (stats.total_pause_seconds / 3600.0).floor() as u32;
    let pause_minutes = ((stats.total_pause_seconds % 3600.0) / 60.0).floor() as u32;
    let pause_seconds = (stats.total_pause_seconds % 60.0).floor() as u32;

    println!("  Total duration:    {}h {}m {}s", hours, minutes, seconds);
    println!("  Total pauses:      {}h {}m {}s", pause_hours, pause_minutes, pause_seconds);
    println!("  Number of pauses:  {}", stats.pause_count);
    println!("  Avg pause length:  {:.2}s", stats.total_pause_seconds / (stats.pause_count as f64).max(1.0));
    println!("  Pause ratio:       {:.1}%", (stats.total_pause_seconds / stats.total_duration_seconds * 100.0).max(0.0));
}

fn vtt_to_txt(vtt_file: &PathBuf) -> Result<String> {
    let content = fs::read_to_string(vtt_file)?;
    let mut text = String::new();
    let mut in_timestamp = false;

    for line in content.lines() {
        if line.contains("-->") {
            in_timestamp = true;
        } else if in_timestamp && !line.is_empty() && !line.starts_with("WEBVTT") {
            text.push_str(line);
            text.push('\n');
            in_timestamp = false;
        }
    }

    Ok(text)
}
