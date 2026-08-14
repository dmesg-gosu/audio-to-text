use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs;
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

    // Build whisper command with vtt output for parsing
    let mut cmd = Command::new("whisper");
    cmd.arg(input_file.to_str().unwrap())
        .arg("--model")
        .arg(&args.model)
        .arg("--output_format")
        .arg("vtt")
        .arg("--output_dir")
        .arg(output_dir.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Add diarization if requested
    if args.diarize {
        cmd.arg("--diarize");
    }

    // Add language if specified
    if let Some(lang) = &args.language {
        cmd.arg("--language").arg(lang);
    }

    // Execute whisper
    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow!("Whisper transcription failed"));
    }

    // Read the generated VTT file to calculate statistics
    let mut vtt_path = input_file.file_stem().unwrap().to_os_string();
    vtt_path.push(".vtt");
    let vtt_file = output_dir.join(&vtt_path);

    if !vtt_file.exists() {
        return Err(anyhow!("VTT output file not created: {:?}", vtt_file));
    }

    // Parse VTT and calculate statistics
    println!("\n\n📊 Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let stats = parse_vtt_and_calculate_stats(&vtt_file)?;
    print_statistics(&stats);

    // Convert VTT to TXT
    let txt_content = vtt_to_txt(&vtt_file)?;

    // Determine final output path
    let final_output = if has_custom_output {
        output_file_name.clone()
    } else {
        let mut path = input_file.file_stem().unwrap().to_os_string();
        path.push(".txt");
        output_dir.join(path)
    };

    // If custom output specified and different from default, move file
    if has_custom_output {
        let default_txt = {
            let mut path = input_file.file_stem().unwrap().to_os_string();
            path.push(".txt");
            output_dir.join(path)
        };
        if default_txt != final_output && default_txt.exists() {
            fs::rename(&default_txt, &final_output)?;
        }
    }

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
    pauses: Vec<(f64, f64)>, // (gap_start, gap_duration)
}

fn parse_vtt_and_calculate_stats(vtt_file: &PathBuf) -> Result<VttStats> {
    let content = fs::read_to_string(vtt_file)?;
    let mut timestamps = Vec::new();
    let mut pauses = Vec::new();
    let mut total_pause_seconds = 0.0;

    // Extract all timestamps
    for line in content.lines() {
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() == 2 {
                let start = time_to_seconds(parts[0].trim());
                let end = time_to_seconds(parts[1].trim());
                timestamps.push((start, end));
            }
        }
    }

    // Calculate pauses between timestamps
    for i in 1..timestamps.len() {
        let gap_start = timestamps[i - 1].1;
        let gap_end = timestamps[i].0;
        let gap_duration = gap_end - gap_start;

        // Only count pauses longer than 0.5 seconds
        if gap_duration > 0.5 {
            total_pause_seconds += gap_duration;
            pauses.push((gap_start, gap_duration));
        }
    }

    let total_duration = if !timestamps.is_empty() {
        timestamps[timestamps.len() - 1].1
    } else {
        0.0
    };

    Ok(VttStats {
        total_duration_seconds: total_duration,
        total_pause_seconds,
        pause_count: pauses.len(),
        pauses,
    })
}

fn time_to_seconds(time_str: &str) -> f64 {
    let trimmed = time_str.trim();
    let parts: Vec<&str> = trimmed.split(':').collect();
    
    if parts.len() == 3 {
        // Format: HH:MM:SS.mmm
        let hours = parts[0].parse::<f64>().unwrap_or(0.0);
        let minutes = parts[1].parse::<f64>().unwrap_or(0.0);
        let seconds_str = parts[2].replace(',', ".");
        let seconds = seconds_str.parse::<f64>().unwrap_or(0.0);
        hours * 3600.0 + minutes * 60.0 + seconds
    } else if parts.len() == 2 {
        // Format: MM:SS.mmm (Whisper VTT format)
        let minutes = parts[0].parse::<f64>().unwrap_or(0.0);
        let seconds_str = parts[1].replace(',', ".");
        let seconds = seconds_str.parse::<f64>().unwrap_or(0.0);
        minutes * 60.0 + seconds
    } else {
        0.0
    }
}

fn format_timestamp(time_str: &str) -> String {
    let total_seconds = time_to_seconds(time_str);
    let minutes = (total_seconds / 60.0).floor() as u32;
    let seconds = (total_seconds % 60.0).floor() as u32;
    
    format!("({:02}:{:02})", minutes, seconds)
}

fn seconds_to_timestamp(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    
    format!("{:02}:{:02}", minutes, secs)
}

fn print_statistics(stats: &VttStats) {
    let hours = (stats.total_duration_seconds / 3600.0).floor() as u32;
    let minutes = ((stats.total_duration_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = (stats.total_duration_seconds % 60.0).floor() as u32;

    let pause_hours = (stats.total_pause_seconds / 3600.0).floor() as u32;
    let pause_minutes = ((stats.total_pause_seconds % 3600.0) / 60.0).floor() as u32;
    let pause_seconds = (stats.total_pause_seconds % 60.0).floor() as u32;

    let avg_pause = if stats.pause_count > 0 {
        stats.total_pause_seconds / stats.pause_count as f64
    } else {
        0.0
    };

    let pause_ratio = if stats.total_duration_seconds > 0.0 {
        (stats.total_pause_seconds / stats.total_duration_seconds) * 100.0
    } else {
        0.0
    };

    println!("  Total duration:    {}h {}m {}s", hours, minutes, seconds);
    println!("  Total pause time:  {}h {}m {}s", pause_hours, pause_minutes, pause_seconds);
    println!("  Number of pauses:  {}", stats.pause_count);
    println!("  Avg pause length:  {:.2}s", avg_pause);
    println!("  Pause ratio:       {:.1}%", pause_ratio);

    if stats.pause_count > 0 && stats.pause_count <= 10 {
        println!("\n  Pause timeline:");
        for (i, (start, duration)) in stats.pauses.iter().enumerate() {
            println!("    Pause {}: {} ({:.2}s)", i + 1, seconds_to_timestamp(*start), duration);
        }
    } else if stats.pause_count > 10 {
        println!("\n  Top 10 longest pauses:");
        let mut sorted_pauses = stats.pauses.clone();
        sorted_pauses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (i, (start, duration)) in sorted_pauses.iter().take(10).enumerate() {
            println!("    Pause {}: {} ({:.2}s)", i + 1, seconds_to_timestamp(*start), duration);
        }
    }
}

fn vtt_to_txt(vtt_file: &PathBuf) -> Result<String> {
    let content = fs::read_to_string(vtt_file)?;
    let mut text = String::new();
    let mut current_timestamp = String::new();

    for line in content.lines() {
        // Skip header
        if line.starts_with("WEBVTT") || line.starts_with("NOTE") {
            continue;
        }

        // Capture timestamps
        if line.contains("-->") {
            let parts: Vec<&str> = line.split("-->").collect();
            if parts.len() >= 1 {
                let start_time = parts[0].trim();
                // Convert MM:SS.mmm to (MM:SS) format
                current_timestamp = format_timestamp(start_time);
            }
            continue;
        }

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Add text with timestamp
        if !current_timestamp.is_empty() {
            text.push_str(&current_timestamp);
            text.push(' ');
            text.push_str(line);
            text.push('\n');
            current_timestamp.clear();
        }
    }

    Ok(text.trim().to_string())
}
