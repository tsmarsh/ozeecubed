# OzeeCubed GUI v2 - Pure winit + wgpu Implementation

This is a ground-up rewrite of the OzeeCubed GUI using pure **winit** + **wgpu** for maximum control and multi-window support.

## Status

✅ **Phase 1 Complete**: Core architecture implemented and compiling successfully!

### Implemented
- Window management with winit 0.30
- Base wgpu renderer with GPU pipeline
- Waveform rendering with persistence effect
- Grid rendering (10x8 divisions)
- Application state management
- Keyboard controls
- Audio capture integration

### Architecture

```
gui-v2/
├── src/
│   ├── main.rs              # Application entry point (ApplicationHandler)
│   ├── window.rs            # Multi-window manager
│   ├── state.rs             # Application state (oscilloscope logic)
│   ├── renderer/
│   │   ├── mod.rs          # Base wgpu renderer
│   │   └── waveform.rs     # Waveform GPU rendering
│   └── shaders/
│       └── waveform.wgsl   # WGSL shader for waveforms
├── Cargo.toml
└── README.md
```

### Key Features

- **Pure wgpu rendering**: All graphics rendered directly on GPU
- **Multi-window ready**: Architecture supports multiple independent windows
- **No UI framework overhead**: Complete control over every pixel
- **Waveform persistence**: Classic oscilloscope phosphor decay effect
- **60 FPS updates**: Real-time audio visualization

### Keyboard Controls

- **Arrow Left/Right**: Adjust time/division
- **Arrow Up/Down**: Adjust volts/division
- **T**: Toggle trigger on/off
- **[ / ]**: Adjust trigger level

### Building

```bash
cargo build -p ozeecubed-gui-v2 --release
```

### Running

```bash
cargo run -p ozeecubed-gui-v2 --release
```

## Next Steps

### Phase 2: Spectrum Analyzer
- Port spectrum rendering to wgpu
- FFT visualization with dB scale
- Frequency labels

### Phase 3: UI System
- Simple button widget
- Slider widget (for controls)
- Text rendering
- Layout system

### Phase 4: Multi-Window
- Spawn additional windows
- Window type selection (waveform, spectrum, XY, waterfall)
- Share audio data between windows

### Phase 5: Advanced Features
- Dockable windows
- Measurements display
- Signal generation
- Export capabilities

## Technical Details

### Dependencies
- `winit 0.30`: Cross-platform windowing
- `wgpu 0.19`: GPU rendering
- `ozeecubed_core`: Shared oscilloscope logic
- `rustfft`: FFT for spectrum analysis

### Rendering Pipeline
1. Clear background to black
2. Draw grid (green lines, 30% alpha)
3. Draw waveform history with persistence (alpha fade)
4. Draw current waveform (full brightness)

### Performance
- GPU-accelerated rendering
- Minimal CPU overhead
- 60 FPS target with ~16ms frame time
- Lock-free audio buffer

## Migration from Iced

This version removes all Iced dependencies and builds everything from scratch:

**Removed**:
- Iced application framework
- Iced Canvas API
- Iced widgets
- Message passing architecture

**Replaced With**:
- Direct winit event loop
- Custom wgpu render pipelines
- Manual UI primitives
- Direct state mutation

**Kept**:
- All core oscilloscope logic (`ozeecubed_core`)
- Audio capture
- Waveform calculations
- Trigger detection
- Persistence history management

## Why winit + wgpu?

1. **Multi-window support**: Native support for multiple windows
2. **Maximum performance**: No framework overhead
3. **Full control**: Every pixel, every interaction
4. **Professional feel**: Build tool-like UIs, not app-like UIs
5. **Future-proof**: Foundation for advanced features (plugins, scripting, etc.)

## License

MIT OR Apache-2.0
