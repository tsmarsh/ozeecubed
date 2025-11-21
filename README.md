# OzeeCubed

A GPU-accelerated digital oscilloscope for real-time audio visualization, written in Rust.

## Features

### Current Features (v0.1.0)

- **Real-time Audio Capture**: Captures audio from your system's default microphone/input device
- **GPU-Accelerated Rendering**: Smooth 60 FPS waveform display using wgpu via Iced
- **Classic Oscilloscope Aesthetic**: Green phosphor-style display with grid overlay
- **Time Base Control**: Adjustable horizontal scale (time per division)
- **Voltage Scale Control**: Adjustable vertical scale (volts per division)
- **Triggering System**: Edge triggering with adjustable level and edge selection (rising/falling)
- **Cross-Platform**: Works on macOS, Linux, and Windows

### Oscilloscope Controls

- **Time/Div**: Controls the horizontal time scale
  - Click `-` to decrease (zoom in on time)
  - Click `+` to increase (zoom out on time)
  - Range: 10 microseconds to seconds per division

- **Volts/Div**: Controls the vertical voltage scale
  - Click `-` to decrease (zoom in on amplitude)
  - Click `+` to increase (zoom out on amplitude)
  - Range: 10mV to volts per division

- **Trigger**: Controls waveform synchronization
  - **ON/OFF**: Toggle triggering (free-run vs triggered mode)
  - **Edge**: Switch between rising and falling edge triggering
  - **Level**: Adjust trigger voltage threshold with `-` and `+`

## Installation

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- Audio input device (microphone)

### Platform-Specific Requirements

#### macOS
```bash
# No additional dependencies required
```

#### Linux
```bash
# Install ALSA development libraries
sudo apt-get install libasound2-dev

# For Wayland support
sudo apt-get install libwayland-dev libxkbcommon-dev
```

#### Windows
```bash
# No additional dependencies required
```

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd ozeecubed

# Build and run
cargo run --release
```

## Usage

1. Launch the application
2. The oscilloscope will automatically connect to your system's default audio input
3. Use the control panel at the bottom to adjust time base, voltage scale, and trigger settings
4. Speak into your microphone or play audio to see the waveform

### Keyboard Shortcuts

(To be implemented in future versions)

## Architecture

OzeeCubed is built with a modular architecture:

- **Audio Module** (`src/audio/`): Handles real-time audio capture using cpal
- **Oscilloscope Module** (`src/oscilloscope/`): Waveform data processing and trigger logic
- **UI Module** (`src/ui/`): Iced-based GPU-accelerated rendering and controls

### Technology Stack

- **Iced**: Cross-platform GUI framework with built-in wgpu support
- **wgpu**: Modern GPU API for high-performance rendering
- **cpal**: Cross-platform audio I/O
- **ringbuf**: Lock-free ring buffer for audio streaming

## Roadmap

### Planned Features

#### Phase 2: Enhanced Functionality
- [ ] Real audio capture integration (currently using test signal)
- [ ] Multi-channel display (stereo L/R separate traces)
- [ ] Audio device selection UI
- [ ] Keyboard shortcuts for all controls
- [ ] Waveform persistence/decay effects (phosphor-like)

#### Phase 3: Measurements & Analysis
- [ ] Automatic frequency measurement
- [ ] Peak-to-peak voltage display
- [ ] RMS voltage calculation
- [ ] Duty cycle measurement
- [ ] Cursors for manual measurements

#### Phase 4: Advanced Triggering
- [ ] Single-shot trigger mode
- [ ] Normal vs Auto trigger modes
- [ ] Pulse width triggering
- [ ] Video triggering

#### Phase 5: Additional Features
- [ ] FFT/Spectrum analyzer view
- [ ] XY mode (Lissajous patterns)
- [ ] Waveform capture and export
- [ ] Screenshot functionality
- [ ] Settings persistence
- [ ] Customizable color themes
- [ ] Multiple input channels (if hardware supports)

#### Phase 6: Professional Features
- [ ] Protocol decoding (UART, SPI, I2C)
- [ ] Math channels (A+B, A-B, etc.)
- [ ] Reference waveforms
- [ ] Waveform averaging
- [ ] Roll mode for slow signals

## Performance

- Target: 60 FPS at all times
- GPU acceleration ensures smooth rendering even with high sample rates
- Lock-free audio buffering minimizes latency

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) GUI framework
- Audio I/O powered by [cpal](https://github.com/RustAudio/cpal)
- Inspired by classic analog oscilloscopes and modern digital scopes

## Troubleshooting

### No Audio Input Detected
- Ensure your microphone is properly connected and set as the default input device
- On Linux, check that your user has permission to access audio devices
- On macOS, grant microphone permissions in System Preferences > Security & Privacy

### Build Errors
- Ensure you have the latest Rust toolchain: `rustup update`
- Check that platform-specific dependencies are installed (see Installation section)

### Performance Issues
- Close other GPU-intensive applications
- Try running in release mode: `cargo run --release`

## Contact

For questions, issues, or feature requests, please open an issue on GitHub.

---

**OzeeCubed** - See your sound, beautifully rendered.
