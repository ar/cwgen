use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use rand::seq::SliceRandom;
use rodio::{OutputStream, Sink};
use std::io::Write;

use crate::morse::{Timing, PracticeMode, text_to_morse, MorseError};
use crate::audio::{play_audio, MorseAudio, NoiseSource, ToneShape};
use crate::OutputMode;

const PRACTICE_SAMPLE_RATE: u32 = 44100;

// ---------- Interactive mode ----------------------------------------------
pub fn interactive_mode(
    timing: Timing,
    tone: u32,
    output: OutputMode,
    qrm: u8,
    tone_shape: ToneShape,
) -> Result<()> {
    println!("Interactive mode – type away (Esc to quit):\n");

    let mut buf = String::new();

    terminal::enable_raw_mode()?;
    let result = (|| {
    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    buf.clear();
                    buf.push(c);

                    match output {
                        OutputMode::Text => {
                            match text_to_morse(&buf) {
                                Ok(morse) => print!("\r\n{}\r\n", morse),
                                Err(e) => print!("\r\nError: {}\r\n", e),
                            }
                        }
                        OutputMode::Audio => {
                            if let Err(e) = play_audio(&buf, timing, tone, qrm, tone_shape, None) {
                                print!("\r\nAudio error: {}\r\n", e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
    })();
    terminal::disable_raw_mode()?;
    result
}

// ---------- Practice mode ----------------------------------------------
pub fn practice_mode(
    initial_wpm: u32,
    gap_ms: u64,
    farnsworth: Option<u32>,
    tone: u32,
    mode: PracticeMode,
    custom_text: Option<&str>,
    qrm: u8,
    tone_shape: ToneShape,
) -> Result<()> {
    let mut content = mode.get_content(custom_text);
    content.shuffle(&mut rand::rng());

    println!("Practice mode – {} words available", content.len());
    println!("Press Space for next, J/← for previous, R to repeat, ↑/↓ to adjust WPM, ? to reveal, Esc to quit:\n");

    let mut current_index = 0;
    let mut current_word = &content[current_index];
    let mut wpm = initial_wpm;
    // Farnsworth requires char_speed > overall_speed, so cap overall WPM below the char speed.
    let max_wpm = farnsworth.map(|f| f.saturating_sub(1)).unwrap_or(100).min(100);
    let mut timing = build_timing(wpm, gap_ms, farnsworth);

    // Persistent audio: a continuous QRM sink runs across the entire session
    // so the noise floor never drops between words, repeats, or WPM changes.
    // The tone sink receives a fresh signal-only buffer for each word and gets
    // mixed against the noise by rodio.
    let (_stream, handle) = OutputStream::try_default()
        .map_err(|e| MorseError::AudioDeviceError(e.to_string()))?;
    let noise_sink = Sink::try_new(&handle)
        .map_err(|e| MorseError::AudioDeviceError(e.to_string()))?;
    noise_sink.append(NoiseSource::new(qrm, PRACTICE_SAMPLE_RATE));
    let tone_sink = Sink::try_new(&handle)
        .map_err(|e| MorseError::AudioDeviceError(e.to_string()))?;

    terminal::enable_raw_mode()?;
    let result = (|| {
    loop {
        tone_sink.append(MorseAudio::new_signal_only(
            PRACTICE_SAMPLE_RATE,
            current_word,
            timing,
            tone,
            tone_shape,
            None,
        ));
        tone_sink.sleep_until_end();

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(' ') => {
                    print!("{} ", current_word);
                    let _ = std::io::stdout().flush();
                    current_index = (current_index + 1) % content.len();
                    current_word = &content[current_index];
                }
                KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Left => {
                    current_index = if current_index == 0 {
                        content.len() - 1
                    } else {
                        current_index - 1
                    };
                    current_word = &content[current_index];
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {}
                KeyCode::Up => {
                    wpm = (wpm + 5).min(max_wpm);
                    timing = build_timing(wpm, gap_ms, farnsworth);
                    print!("({}wpm) ", wpm);
                    let _ = std::io::stdout().flush();
                }
                KeyCode::Down => {
                    wpm = wpm.saturating_sub(5).max(1);
                    timing = build_timing(wpm, gap_ms, farnsworth);
                    print!("({}wpm) ", wpm);
                    let _ = std::io::stdout().flush();
                }
                KeyCode::Char('?') => {
                    print!("[{}]", current_word);
                    let _ = std::io::stdout().flush();
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
    })();
    terminal::disable_raw_mode()?;
    result
}

fn build_timing(wpm: u32, gap_ms: u64, farnsworth: Option<u32>) -> Timing {
    match farnsworth {
        Some(char_speed) => Timing::new_farnsworth(char_speed, wpm, gap_ms),
        None => Timing::new(wpm, gap_ms),
    }
}

