# SSTV Processor

A Rust application for processing images through SSTV (Slow Scan Television) encoding/decoding with noise and retarder effects.

## Features

- Real-time GUI with live preview
- Command-line interface
- SSTV Martin M1 mode (320×256)
- AWGN noise generation with envelope modulation
- Retarder (ghost image) effects with delay
- Auto-scaling for any image size

## Quick Start

```bash
# Build
cargo build --release

# Run GUI
cargo run --bin gui

# Run CLI
cargo run --bin cli -- -i input.jpg -o output.png
```

## GUI Usage

1. Load main image with "Выбрать" button
2. Optionally load retarder image
3. Adjust noise and retarder parameters

### Processing Modes

- **Fast Mode**: Instant preview (~0.1s) when no effects applied
- **SSTV Mode**: Full SSTV processing (~30-40s) with noise or retarder effects

## CLI Parameters

```bash
cargo run --bin cli -- [OPTIONS]

Options:
  -i, --input <FILE>          Input image (PNG/JPG)
  -o, --output <FILE>         Output file [default: output.png]
  -n, --noise <0-100>         Noise level [default: 0]
  --noise-env <ENVELOPE>      Noise envelope [default: const]
  --noise-repeat <FLOAT>      Noise repetition [default: 1.0]
  -r, --retarder <FILE>       Retarder image
  --level <0.0-1.0>          Retarder level [default: 0.3]
  --ret-env <ENVELOPE>        Retarder envelope [default: const]
  --ret-repeat <FLOAT>        Retarder repetition [default: 1.0]
  --delay-ms <MS>            Retarder delay [default: 0]
```

### Envelope Types
- `const` - Constant level
- `sin` - Sine wave
- `tri` - Triangle wave
- `saw` - Sawtooth wave
- `square` - Square wave
- `rand` - Random

## Examples

```bash
# Basic SSTV processing
cargo run --bin cli -- -i photo.jpg -o result.png

# Add noise
cargo run --bin cli -- -i photo.jpg -n 25 --noise-env sin -o noisy.png

# Retarder effect
cargo run --bin cli -- -i main.jpg -r overlay.jpg --level 0.4 -o mixed.png

# Complex processing
cargo run --bin cli -- -i input.jpg -r retarder.jpg \
  -n 30 --noise-env tri --level 0.3 --delay-ms 150 -o output.png
```

## Technical Details

- **SSTV Mode**: Martin M1 (320×256, 11025 Hz)
- **Processing**: Auto-converts images to SSTV resolution
- **Output**: Results scaled back to original dimensions
- **Performance**: Fast mode for previews, SSTV mode for authentic artifacts

## Architecture

```
src/
├── lib.rs          # Library exports
├── envelope.rs     # Envelope functions
├── noise.rs        # Noise processor
├── retarder.rs     # Retarder processor
├── processor.rs    # Main SSTV processor
└── bin/
    ├── cli.rs      # Command-line interface
    └── gui.rs      # GUI interface
```

## Building

```bash
cargo build --release
```

## Dependencies

- `image` - Image processing
- `rsstv` - SSTV encoding/decoding
- `eframe` - GUI framework
- `clap` - CLI parsing
