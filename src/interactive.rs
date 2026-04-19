use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal;
use std::io::Write;

use crate::morse::{Timing, PracticeMode, text_to_morse};
use crate::audio::{play_audio, ToneShape};
use crate::OutputMode;

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
    timing: Timing, 
    tone: u32, 
    mode: PracticeMode, 
    custom_text: Option<&str>,
    qrm: u8,
    tone_shape: ToneShape,
) -> Result<()> {
    let content = mode.get_content(custom_text);
    
    println!("Practice mode – {} words available", content.len());
    println!("Press Space for next, R to repeat, ? to reveal, Esc to quit:\n");

    let mut current_index = 0;
    let mut current_word = &content[current_index];

    terminal::enable_raw_mode()?;
    let result = (|| {
    loop {
        if let Err(e) = play_audio(current_word, timing, tone, qrm, tone_shape, None) {
            print!("Audio error: {}\r\n", e);
        }

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(' ') => {
                    print!("{} ", current_word);
                    let _ = std::io::stdout().flush();
                    current_index = (current_index + 1) % content.len();
                    current_word = &content[current_index];
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {}
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

