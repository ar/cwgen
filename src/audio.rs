use anyhow::Result;
use hound::{WavSpec, WavWriter};
use rand::Rng;
use rodio::{source::Source, OutputStream, Sink};
use std::time::Duration;

use crate::morse::{Timing, MorseError};

// ---------- Tone Generator -------------------------------------------------
pub struct ToneGenerator {
    sample_rate: u32,
    base_frequency: f64,
    current_frequency: f64,
    phase: f64,
    shape: ToneShape,
    drift_percentage: Option<u8>,
    symbol_start_time: f64,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ToneShape {
    Sine,
    Square,
    Sawtooth,
}

impl ToneGenerator {
    pub fn new(frequency: u32, sample_rate: u32, shape: ToneShape, drift_percentage: Option<u8>) -> Self {
        Self {
            sample_rate,
            base_frequency: frequency as f64,
            current_frequency: frequency as f64,
            phase: 0.0,
            shape,
            drift_percentage,
            symbol_start_time: 0.0,
        }
    }
    
    pub fn start_symbol(&mut self, sample_time: f64) {
        if self.drift_percentage.is_some() {
            self.symbol_start_time = sample_time;
            self.current_frequency = self.base_frequency;
        }
        // Reset phase to prevent discontinuities at symbol start
        self.phase = 0.0;
    }
    
    pub fn next_sample(&mut self, sample_time: f64) -> f32 {
        if let Some(drift_pct) = self.drift_percentage {
            // Calculate frequency drift based on time into current symbol
            let time_in_symbol = sample_time - self.symbol_start_time;
            
            // Convert percentage to fraction (e.g., 75 -> 0.75)
            let target_fraction = drift_pct as f64 / 100.0;
            
            // Exponential decay: start at base frequency, drift down to target fraction
            // Faster decay for more dramatic effect
            let decay_rate = 1.2; // Higher = faster drift
            let drift_factor = target_fraction + (1.0 - target_fraction) * (-decay_rate * time_in_symbol).exp();
            self.current_frequency = self.base_frequency * drift_factor;
        }
        
        let increment = 2.0 * std::f64::consts::PI * self.current_frequency / self.sample_rate as f64;
        self.phase += increment;
        if self.phase > 2.0 * std::f64::consts::PI {
            self.phase -= 2.0 * std::f64::consts::PI;
        }
        
        match self.shape {
            ToneShape::Sine => self.phase.sin() as f32,
            ToneShape::Square => {
                if self.phase < std::f64::consts::PI { 0.8 } else { -0.8 }
            }
            ToneShape::Sawtooth => {
                (self.phase / (2.0 * std::f64::consts::PI) * 2.0 - 1.0) as f32 * 0.8
            }
        }
    }
}

// ---------- SSB-style band-pass noise --------------------------------------
struct SsbNoise {
    amplitude: f32,
    i: f32,
    q: f32,
    phase: f64,
}

impl SsbNoise {
    fn new(qrm_level: u8) -> Self {
        // Calibrated QRM levels based on amateur radio S-meter scale
        // Signal is considered S9 (strong), noise levels are relative to that
        let noise_amplitude = match qrm_level {
            0 => 0.01,   // S1 - barely audible noise
            1 => 0.03,   // S2 - very light noise
            2 => 0.06,   // S3 - light noise
            3 => 0.10,   // S4 - moderate noise
            4 => 0.18,   // S5 - noticeable noise, but easy copy
            5 => 0.30,   // S6 - moderate interference
            6 => 0.50,   // S7 - significant interference
            7 => 0.80,   // S8 - difficult copy conditions
            8 => 1.20,   // S9+10dB - very difficult
            9 => 2.00,   // S9+20dB - extremely difficult, near impossible
            _ => 0.01,   // fallback
        };
        
        SsbNoise {
            amplitude: noise_amplitude,
            i: 0.0,
            q: 0.0,
            phase: 0.0,
        }
    }

    fn next(&mut self, sample_rate: u32) -> f32 {
        // 1. wide-band white
        let white = rand::rng().random_range(-1.0f32..1.0);
        // 2. very gentle low-pass (≈ 3 kHz)  -- I branch
        self.i += (white - self.i) * 0.12;
        // 3. shift +90° via Hilbert-ish (Q branch)
        let target_q = self.i;
        self.q += (target_q - self.q) * 0.12;
        // 4. complex multiply by +USB carrier (1 kHz inside pass-band)
        self.phase += 2.0 * std::f64::consts::PI * 1000.0 / sample_rate as f64;
        let car_i = self.phase.cos() as f32;
        let car_q = self.phase.sin() as f32;
        let usb = self.i * car_i - self.q * car_q;  // upper side-band only
        // 5. Apply calibrated amplitude
        usb * self.amplitude
    }
}

// ---------- Audio generator ------------------------------------------------
pub struct MorseAudio {
    samples: Vec<f32>,
    pos: usize,
    sample_rate: u32,
}

impl MorseAudio {
    pub fn new_with_sample_rate(
        sample_rate: u32,
        text: &str, 
        timing: Timing, 
        tone: u32, 
        qrm: u8,
        tone_shape: ToneShape,
        drift_percentage: Option<u8>,
    ) -> Self {
        let mut tone_generator = ToneGenerator::new(tone, sample_rate, tone_shape, drift_percentage);
        let mut samples = Vec::new();
        let mut noise = SsbNoise::new(qrm);

        let attack_dur  = timing.sym.mul_f32(0.15);
        let release_dur = timing.sym.mul_f32(0.25);

        // Morse signal amplitude (S9 level)
        let signal_amplitude = 0.25;
        
        let mut sample_time = 0.0;
        let mut is_first_symbol = true;

        // Build tone track - noise should be continuous throughout
        for ch in text.chars() {
            let up = ch.to_ascii_uppercase();
            if let Some(code) = crate::morse::MORSE.get(&up) {
                for sym in code.chars() {
                    let dur = match sym { 
                        '.' => timing.dot, 
                        '-' => timing.dash, 
                        _ => continue 
                    };
                    
                    let len = (sample_rate as f64 * dur.as_secs_f64()) as usize;
                    let attack  = (sample_rate as f64 * attack_dur.as_secs_f64()) as usize;
                    let release = (sample_rate as f64 * release_dur.as_secs_f64()) as usize;
                    
                    // Start new symbol - reset frequency for drift and phase for continuity
                    tone_generator.start_symbol(sample_time);
                    
                    // Generate tone with envelope PLUS continuous noise
                    for i in 0..len {
                        let mut amp = 1.0;
                        if i < attack { 
                            amp = i as f32 / attack as f32; 
                        }
                        if i >= len - release { 
                            amp = (len - i) as f32 / release as f32; 
                        }
                        
                        // Extra gentle start for the very first symbol to prevent any click
                        if is_first_symbol && i == 0 {
                            amp *= 0.1;
                        }
                        
                        let tone_sample = tone_generator.next_sample(sample_time) * signal_amplitude * amp;
                        let noise_sample = noise.next(sample_rate);
                        samples.push(tone_sample + noise_sample);
                        sample_time += 1.0 / sample_rate as f64;
                    }
                    
                    is_first_symbol = false;
                    
                    // Symbol space - continuous noise only (no tone)
                    let off = (sample_rate as f64 * timing.sym.as_secs_f64()) as usize;
                    for _ in 0..off {
                        samples.push(noise.next(sample_rate)); // Full noise during gaps
                        sample_time += 1.0 / sample_rate as f64;
                    }
                }
                
                // Character space - continuous noise only (no tone)
                let off = (sample_rate as f64 * (timing.chr - timing.sym).as_secs_f64()) as usize;
                for _ in 0..off {
                    samples.push(noise.next(sample_rate)); // Full noise during gaps
                    sample_time += 1.0 / sample_rate as f64;
                }
            } else if up == ' ' {
                // Word space - continuous noise only (no tone)
                let off = (sample_rate as f64 * (timing.wrd - timing.chr).as_secs_f64()) as usize;
                for _ in 0..off {
                    samples.push(noise.next(sample_rate)); // Full noise during gaps
                    sample_time += 1.0 / sample_rate as f64;
                }
            }
        }

        MorseAudio {
            samples,
            pos: 0,
            sample_rate,
        }
    }

    pub fn new(
        text: &str, 
        timing: Timing, 
        tone: u32, 
        qrm: u8,
        tone_shape: ToneShape,
        drift_percentage: Option<u8>,
    ) -> Self {
        // Use 44100 Hz for high-quality audio playback
        Self::new_with_sample_rate(44100, text, timing, tone, qrm, tone_shape, drift_percentage)
    }

    pub fn get_samples(&self) -> &[f32] {
        &self.samples
    }
}

impl Iterator for MorseAudio {
    type Item = f32;
    
    fn next(&mut self) -> Option<f32> {
        if self.pos < self.samples.len() {
            let sample = self.samples[self.pos];
            self.pos += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for MorseAudio {
    fn current_frame_len(&self) -> Option<usize> { None }
    
    fn channels(&self) -> u16 { 1 }
    
    fn sample_rate(&self) -> u32 { self.sample_rate }
    
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(
            self.samples.len() as f64 / self.sample_rate as f64
        ))
    }
}

// ---------- Audio playback helper ------------------------------------------
pub fn play_audio(
    text: &str, 
    timing: Timing, 
    tone: u32, 
    qrm: u8,
    tone_shape: ToneShape,
    drift_percentage: Option<u8>,
) -> Result<()> {
    let (_stream, handle) = OutputStream::try_default()
        .map_err(|e| MorseError::AudioDeviceError(e.to_string()))?;
    
    let sink = Sink::try_new(&handle)
        .map_err(|e| MorseError::AudioDeviceError(e.to_string()))?;
    
    sink.append(MorseAudio::new(text, timing, tone, qrm, tone_shape, drift_percentage));
    sink.sleep_until_end();
    
    Ok(())
}

// ---------- WAV file output ------------------------------------------------
pub fn save_audio_to_wav(
    text: &str,
    timing: Timing,
    tone: u32,
    qrm: u8,
    tone_shape: ToneShape,
    drift_percentage: Option<u8>,
    filename: &str,
) -> Result<()> {
    // Use 8000 Hz for smaller WAV files - adequate for morse code
    let morse_audio = MorseAudio::new_with_sample_rate(8000, text, timing, tone, qrm, tone_shape, drift_percentage);
    let samples = morse_audio.get_samples();
    
    let spec = WavSpec {
        channels: 1,
        sample_rate: morse_audio.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(filename, spec)?;
    
    for &sample in samples {
        // Convert f32 sample in range [-1.0, 1.0] to i16
        let scaled = (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer.write_sample(scaled)?;
    }
    
    writer.finalize()?;
    Ok(())
}

