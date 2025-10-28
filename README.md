# Morse Code Generator

A Rust-based command-line tool for generating Morse code audio and text with configurable speed, tone, and background noise simulation. Useful for amateur radio operators and Morse code learners.

## Features

- **Multiple Input Modes**: Read from stdin, files, or interactive typing
- **Configurable Speed**: 1-100 WPM (PARIS standard)
- **Tone Control**: 100-3000 Hz frequency with multiple waveform options
- **Realistic QRM**: Background noise simulation with 10 levels of interference
- **Output Options**: Play audio through speakers or save to WAV files
- **Practice Modes**: Random words, callsigns, Q-codes, and numbers
- **Farnsworth Timing**: Learn at high character speeds with extended spacing
- **Interactive Mode**: Real-time typing practice with immediate feedback

## Installation

### Prerequisites

- Rust and Cargo (1.70.0 or later)
- System audio output (for audio playback)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/ar/cwgen.git
cd cwgen

# Build in release mode
cargo build --release

# The binary will be at ./target/release/cwgen
```



### Installing via Cargo

```bash
cargo install --path .
```

## Usage

### Basic Examples

```bash
# Play Morse code from stdin
echo "HELLO WORLD" | cwgen

# Play from a file
cwgen --file message.txt

# Text output instead of audio
echo "SOS" | cwgen --output text

# Save to WAV file
echo "CQ CQ DE W1AW" | cwgen --output-file transmission.wav
```



### Configuration Options


```bash
# Speed and tone control
cwgen --wpm 15 --tone 800

# Add background noise (QRM levels 0-9)
cwgen --qrm 3

# Different tone shapes
cwgen --tone-shape square
cwgen --tone-shape sawtooth

# Farnsworth timing for learning
cwgen --farnsworth 25 --wpm 15
```



### Interactive Mode

```bash
# Type characters and hear Morse code immediately
cwgen --interactive

# Interactive mode with text output
cwgen --interactive --output text
```



### Practice Modes

```bash
# Practice with random words
cwgen --practice random-words

# Practice amateur radio callsigns
cwgen --practice callsigns

# Practice Q-codes
cwgen --practice qcodes

# Practice numbers
cwgen --practice numbers

# Custom practice text
cwgen --practice custom --custom-text "CQ TEST DE"
```



## Command Line Reference



```bash
USAGE:
    cwgen [OPTIONS]

OPTIONS:
    -f, --file <FILE>              Read text from file instead of stdin
    -h, --help                     Print help information
    -i, --interactive              Interactive typing mode (press Esc to quit)
    -p, --practice <PRACTICE>      Practice mode (random-words, callsigns, qcodes, numbers, custom)
        --custom-text <CUSTOM_TEXT> Custom text for practice mode
    -s, --wpm <WPM>                Speed in WPM (PARIS standard) [default: 20]
    -t, --tone <TONE>              Tone frequency in Hz [default: 700]
    -g, --gap-ms <GAP_MS>          Extra gap between characters in ms [default: 0]
        --output <OUTPUT>          Output mode [default: audio] [possible values: audio, text]
        --qrm <S>                  Background QRM: S0 (no noise) … S9 (extreme) [default: 0]
        --tone-shape <TONE_SHAPE>  Tone shape [default: sine] [possible values: sine, square, sawtooth]
        --farnsworth <FARNSWORTH>  Use Farnsworth timing for learning (specify character speed)
        --output-file <OUTPUT_FILE> Save audio to WAV file instead of playing
        --drift <DRIFT>            Frequency drift percentage (0-100) - simulates homebrew transmitter
    -V, --version                  Print version information
```

## QRM Levels

The `--qrm` parameter simulates realistic radio interference:

- **0-2**: Light noise - Easy copy conditions
- **3-4**: Moderate noise - Good for intermediate practice
- **5-6**: Significant interference - Requires concentration
- **7-8**: Difficult conditions - Expert level
- **9**: Extreme interference - Near impossible copy

## Practice Tips

### For Beginners (5-10 WPM)

```bash
cwgen --practice random-words --wpm 10 --qrm 0
```



### Intermediate (10-20 WPM)

```bash
cwgen --practice callsigns --wpm 15 --qrm 2
```



### Advanced (20+ WPM)


```bash
cwgen --practice qcodes --wpm 25 --qrm 4
```



### Farnsworth Method

Use Farnsworth timing to learn high-speed recognition:


```bash
cwgen --practice random-words --farnsworth 25 --wpm 15
```



## Testing

Run the test suite to verify functionality:

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test morse
cargo test audio

# Run with detailed output
cargo test -- --nocapture
```



## File Output

Save Morse code transmissions to WAV files for later practice:


```bash
# Save a message
echo "THE QUICK BROWN FOX" | cwgen --output-file practice.wav

# Save with specific parameters
echo "CQ CQ DE W1AW" | cwgen --wpm 20 --tone 700 --qrm 3 --output-file qso.wav
```



Generated WAV files use 8000 Hz sample rate for compact file sizes while maintaining clear Morse code reproduction.

## Morse Code Reference

The tool supports standard Morse code characters plus common prosigns:

- Letters: A-Z
- Numbers: 0-9
- Punctuation: . , ? / & ( ) + = @ : ' " !
- Prosigns: `<AA>` (new line), `<AR>` (end), `<AS>` (wait), `<BT>` (break), `<KN>` (invite), `<SK>` (end work)

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## Troubleshooting

### No Audio Output

- Check system audio settings
- Verify audio permissions
- Try using `--output-file` to test audio generation

### Build Issues

- Ensure Rust is up to date: `rustup update`
- Clear cargo cache: `cargo clean`

### Performance

- For long texts, use file output instead of real-time playback
- Lower sample rate (already optimized to 8000 Hz) keeps files small

------

Happy copying! 73 de CX4CC.



