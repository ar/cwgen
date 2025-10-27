use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};

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
                                Ok(morse) => println!("\n{}", morse),
                                Err(e) => eprintln!("\nError: {}", e),
                            }
                        }
                        OutputMode::Audio => {
                            if let Err(e) = play_audio(&buf, timing, tone, qrm, tone_shape) {
                                eprintln!("\nAudio error: {}", e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
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
    println!("Press Space for next, R to repeat, Esc to quit:\n");
    
    let mut current_index = 0;
    let mut current_word = &content[current_index];
    
    loop {
        println!("Current: {}", current_word);
        match text_to_morse(current_word) {
            Ok(morse) => println!("Morse: {}", morse),
            Err(e) => eprintln!("Error: {}", e),
        }
        
        if let Err(e) = play_audio(current_word, timing, tone, qrm, tone_shape) {
            eprintln!("Audio error: {}", e);
        }
        
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(' ') => {
                    // Move to next word, wrap around if at the end
                    current_index = (current_index + 1) % content.len();
                    current_word = &content[current_index];
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Repeat current word
                    println!("Repeating: {}", current_word);
                }
                _ => {}
            },
            _ => {}
        }
    }
    
    Ok(())
}

