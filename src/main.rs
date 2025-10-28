use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::io::Read;

mod morse;
mod audio;
mod interactive;

use morse::{MorseError, Timing, PracticeMode, text_to_morse};
use audio::{play_audio, ToneShape, save_audio_to_wav};
use interactive::{interactive_mode, practice_mode};

// ---------- CLI ------------------------------------------------------------
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Speed in WPM (PARIS standard)
    #[arg(short, long, default_value_t = 20)]
    wpm: u32,

    /// Tone frequency in Hz
    #[arg(short, long, default_value_t = 700)]
    tone: u32,

    /// Extra gap between characters in ms
    #[arg(short, long, default_value_t = 0)]
    gap_ms: u64,

    /// Output mode
    #[arg(long, value_enum, default_value_t = OutputMode::Audio)]
    output: OutputMode,

    /// Read text from file instead of stdin
    #[arg(short, long)]
    file: Option<String>,

    /// Interactive typing mode (press Esc to quit)
    #[arg(short, long)]
    interactive: bool,

    /// Background QRM: S0 (no noise) … S9 (extreme)  (0-9)
    #[arg(long, value_name = "S", default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=9))]
    qrm: u8,

    /// Practice mode (random words, callsigns, Q-codes, numbers)
    #[arg(short, long, value_enum)]
    practice: Option<PracticeMode>,

    /// Custom text for practice mode
    #[arg(long, requires = "practice")]
    custom_text: Option<String>,

    /// Tone shape
    #[arg(long, value_enum, default_value_t = ToneShape::Sine)]
    tone_shape: ToneShape,

    /// Use Farnsworth timing for learning (specify character speed)
    #[arg(long)]
    farnsworth: Option<u32>,

    /// Save audio to WAV file instead of playing
    #[arg(long)]
    output_file: Option<String>,

    /// Frequency drift percentage (0-100) - simulates homebrew transmitter
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
    drift: Option<u8>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputMode {
    Audio,
    Text,
}

// ---------- Text output ----------------------------------------------------
fn print_morse(text: &str) -> Result<()> {
    let morse = text_to_morse(text)?;
    println!("{}", morse);
    Ok(())
}

// ---------- Main -----------------------------------------------------------
fn main() -> Result<()> {
    let args = Args::parse();

    // Validate arguments
    if let Err(e) = validate_args(&args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let timing = if let Some(char_speed) = args.farnsworth {
        Timing::new_farnsworth(char_speed, args.wpm, args.gap_ms)
    } else {
        Timing::new(args.wpm, args.gap_ms)
    };

    // Handle practice mode
    if let Some(mode) = args.practice {
        return practice_mode(
            timing, 
            args.tone, 
            mode, 
            args.custom_text.as_deref(), 
            args.qrm,
            args.tone_shape,
        );
    }

    // Handle interactive mode
    if args.interactive {
        return interactive_mode(timing, args.tone, args.output, args.qrm, args.tone_shape);
    }

    // Read input text
    let text = if let Some(path) = &args.file {
        std::fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    };

    // Process based on output mode
    match args.output {
        OutputMode::Text => print_morse(&text),
        OutputMode::Audio => {
            if let Some(output_path) = &args.output_file {
                // Save to WAV file
                save_audio_to_wav(&text, timing, args.tone, args.qrm, args.tone_shape, args.drift, output_path)?;
                println!("Saved morse code to: {}", output_path);
                Ok(())
            } else {
                // Play audio normally
                play_audio(&text, timing, args.tone, args.qrm, args.tone_shape, args.drift)
            }
        }
    }
}

fn validate_args(args: &Args) -> Result<(), MorseError> {
    if args.wpm < 1 || args.wpm > 100 {
        return Err(MorseError::InvalidSpeed(args.wpm));
    }
    if args.tone < 100 || args.tone > 3000 {
        return Err(MorseError::InvalidTone(args.tone));
    }
    if let Some(farnsworth) = args.farnsworth {
        if farnsworth < 5 || farnsworth > 40 {
            return Err(MorseError::InvalidSpeed(farnsworth));
        }
        if farnsworth <= args.wpm {
            return Err(MorseError::InvalidFarnsworth(farnsworth, args.wpm));
        }
    }
    Ok(())
}

