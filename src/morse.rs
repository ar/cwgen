use lazy_static::lazy_static;
use phf::phf_map;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

// ---------- Error types ----------------------------------------------------
#[derive(Error, Debug)]
pub enum MorseError {
    #[error("Invalid character for morse: '{0}'")]
    InvalidCharacter(char),
    #[error("Invalid speed: {0} WPM (must be 1-100)")]
    InvalidSpeed(u32),
    #[error("Invalid tone: {0} Hz (must be 100-3000)")]
    InvalidTone(u32),
    #[error("Invalid Farnsworth timing: character speed {0} must be greater than overall speed {1}")]
    InvalidFarnsworth(u32, u32),
    #[error("Audio device error: {0}")]
    AudioDeviceError(String),
}

// ---------- Morse table -----------------------------------------------------
pub const MORSE: phf::Map<char, &'static str> = phf_map! {
    'A' => ".-",    'B' => "-...",  'C' => "-.-.",  'D' => "-..",
    'E' => ".",     'F' => "..-.",  'G' => "--.",   'H' => "....",
    'I' => "..",    'J' => ".---",  'K' => "-.-",   'L' => ".-..",
    'M' => "--",    'N' => "-.",    'O' => "---",   'P' => ".--.",
    'Q' => "--.-",  'R' => ".-.",   'S' => "...",   'T' => "-",
    'U' => "..-",   'V' => "...-",  'W' => ".--",   'X' => "-..-",
    'Y' => "-.--",  'Z' => "--..",
    '0' => "-----", '1' => ".----", '2' => "..---", '3' => "...--",
    '4' => "....-", '5' => ".....", '6' => "-....", '7' => "--...",
    '8' => "---..", '9' => "----.",
    '.' => ".-.-.-", ',' => "--..--", '?' => "..--..", '/' => "-..-.",
    '&' => ".-...", '(' => "-.--.",  ')' => "-.--.-", '+' => ".-.-.",
    '=' => "-...-", '@' => ".--.-.", ':' => "---...", '\'' => ".----.",
    '"' => ".-..-.", '!' => "-.-.--", '-' => "-...-",
    ' ' => "/",
    '\n' => "",     // Handle newlines as empty (no morse output)
    '\r' => "",     // Handle carriage returns as empty
};

// ---------- Timing ---------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Timing {
    pub dot: Duration,
    pub dash: Duration,
    pub sym: Duration,
    pub chr: Duration,
    pub wrd: Duration,
}

impl Timing {
    pub fn new(wpm: u32, extra_gap_ms: u64) -> Self {
        let unit = Duration::from_millis(1200 / wpm as u64);
        let extra = Duration::from_millis(extra_gap_ms);
        Timing {
            dot: unit,
            dash: unit * 3,
            sym: unit,
            chr: unit * 3 + extra,
            wrd: unit * 7 + extra,
        }
    }

    pub fn new_farnsworth(char_speed: u32, overall_speed: u32, extra_gap_ms: u64) -> Self {
        let char_unit = Duration::from_millis(1200 / char_speed as u64);
        let overall_unit = Duration::from_millis(1200 / overall_speed as u64);
        let extra = Duration::from_millis(extra_gap_ms);
        
        // Farnsworth: characters at normal speed, extended inter-element spacing
        let extended_gap = overall_unit * 7 - char_unit * (7 - 1); // PARIS has 7 dots worth of gaps
        
        Timing {
            dot: char_unit,
            dash: char_unit * 3,
            sym: char_unit,
            chr: char_unit * 3 + extended_gap + extra,
            wrd: char_unit * 7 + extended_gap * 2 + extra,
        }
    }
}

lazy_static! {
    pub static ref COMMON_TIMINGS: HashMap<u32, Timing> = {
        let mut m = HashMap::new();
        for wpm in 5..=50 {
            m.insert(wpm, Timing::new(wpm, 0));
        }
        m
    };
}

// ---------- Morse Conversion ------------------------------------------------
pub fn text_to_morse(text: &str) -> Result<String, MorseError> {
    let mut morse_string = String::new();
    
    for ch in text.chars() {
        let up = ch.to_ascii_uppercase();
        
        // Handle regular characters
        if let Some(code) = MORSE.get(&up) {
            if !code.is_empty() {  // Skip empty codes (like newlines)
                morse_string.push_str(code);
                morse_string.push(' ');
            }
        } else {
            return Err(MorseError::InvalidCharacter(ch));
        }
    }
    
    Ok(morse_string.trim().to_string())
}

// ---------- Practice Mode Content -------------------------------------------
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum PracticeMode {
    RandomWords,
    Callsigns,
    QCodes,
    Numbers,
    Custom,
}

impl PracticeMode {
    pub fn get_content(&self, custom_text: Option<&str>) -> Vec<String> {
        match self {
            PracticeMode::RandomWords => vec![
                "THE", "QUICK", "BROWN", "FOX", "JUMPS", "OVER", "LAZY", "DOG",
                "PARIS", "CODEX", "MORSE", "HAM", "RADIO", "SIGNAL", "CODE",
            ].iter().map(|s| s.to_string()).collect(),
            PracticeMode::Callsigns => vec![
                "W1AW", "K2ABC", "N3XYZ", "W4DEF", "K5GHI", "N6JKL", 
                "W7MNO", "K8PQR", "N9STU", "VE3ABC", "G4HAM",
            ].iter().map(|s| s.to_string()).collect(),
            PracticeMode::QCodes => vec![
                "QTH", "QRZ", "QSL", "QRM", "QRN", "QRP", "QRQ", "QRS", 
                "QRT", "QRU", "QRV", "QSB", "QSY", "QSO",
            ].iter().map(|s| s.to_string()).collect(),
            PracticeMode::Numbers => vec![
                "123", "456", "789", "012", "345", "678", "901", "234", 
                "567", "890", "73", "88", "55",
            ].iter().map(|s| s.to_string()).collect(),
            PracticeMode::Custom => {
                if let Some(text) = custom_text {
                    text.split_whitespace().map(|s| s.to_string()).collect()
                } else {
                    vec!["CQ", "DE", "TEST"].iter().map(|s| s.to_string()).collect()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_calculation() {
        let timing = Timing::new(20, 0);
        assert_eq!(timing.dot.as_millis(), 60); // 1200 / 20 = 60ms
        assert_eq!(timing.dash.as_millis(), 180); // 3 * 60ms
    }

    #[test]
    fn test_morse_conversion() {
        assert_eq!(text_to_morse("SOS").unwrap(), "... --- ...");
        assert_eq!(text_to_morse("AB").unwrap(), ".- -...");
    }

    #[test]
    fn test_invalid_character() {
        assert!(text_to_morse("SÖS").is_err());
    }

    #[test]
    fn test_newline_handling() {
        assert_eq!(text_to_morse("A\nB").unwrap(), ".- -...");
    }
}

