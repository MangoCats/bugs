# Bugs 0.29 Rust - Architecture Guide

This document provides a technical overview of the Bugs 0.29 Rust implementation architecture.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
├──────────────────┬────────────────┬─────────────────────┤
│   bugs-cli       │ bugs-viewer-   │  bugs-viewer-web    │
│   (headless)     │   native       │  (WebAssembly)      │
│                  │  (wgpu+egui)   │  (Canvas 2D)        │
└────────┬─────────┴────────┬───────┴──────────┬──────────┘
         │                  │                  │
         └──────────────────┼──────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         │         bugs-render                 │
         │  (Visualizer + GraphRenderer)       │
         └──────────────────┬──────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         │         bugs-recorder               │
         │    (Event logging & playback)       │
         └──────────────────┬──────────────────┘
                            │
         ┌──────────────────┴──────────────────┐
         │          bugs-core                  │
         │  (Simulation engine & genetics)     │
         └─────────────────────────────────────┘
```

## Crate Responsibilities

### bugs-core
**Purpose**: Pure simulation logic, no I/O dependencies

**Key Components**:
- `World`: Main simulation container with hexagonal grid
- `Bug`: Individual bug with genetic brain and state
- `BugBrain`: Genetic programming decision system
- `Chromosome`: Diploid gene pairs (A/B chromosomes)
- `Simulation`: Orchestrates tick execution

**Important Constants** (`constants.rs`):
```rust
WORLD_X = 1760        // World width
WORLD_Y = 1000        // World height
LEFTBAR = 80          // Left sidebar width
SIDEBAR = 80          // Total sidebar (LEFTBAR + RIGHTBAR)
BOTTOMBAR = 80        // Bottom graph height
RENDER_WIDTH = 1840   // WORLD_X + SIDEBAR
RENDER_HEIGHT = 1000  // Same as WORLD_Y
```

**Genetics System**:
- 8 decision types (MOVE_OK, TURN_OK, FOOD_OK, etc.)
- Each bug has 2 chromosomes (A and B) per decision
- Expression trees evaluated to make decisions
- Mutations: add gene, remove gene, modify gene

### bugs-recorder
**Purpose**: Event recording and replay with compression

**Key Components**:
- `EventWriter`: Records simulation events
- `EventReader`: Replays from event log
- `Snapshot`: Full world state at specific tick

**File Format**:
- `.events`: Bincode-serialized event stream
- `.snapshots`: LZ4-compressed world states

**Compression**:
- Events: Length-prefixed bincode messages
- Snapshots: LZ4 compression (typically 10:1 ratio)
- Snapshot interval: Configurable (default every 500 ticks)

### bugs-render
**Purpose**: Rendering abstraction, generates RGBA pixel buffers

**Components**:

#### Visualizer
Renders main world view (1840×1000):
```rust
pub struct Visualizer {
    width: usize,   // RENDER_WIDTH (1840)
    height: usize,  // RENDER_HEIGHT (1000)
    mode: VisMode,  // BugMap or EnvironmentMap
}
```

**Rendering Modes**:
- `BugMap`: Bug ethnicity colors with position trails
- `EnvironmentMap`: Food (green), water (blue), bugs (red)

**LEFTBAR Rendering**:
- 80px wide activity visualization
- Row-by-row action distribution
- Color-coded bars per action type:
  - Sleep: Dark gray
  - Eat: Green
  - Move: Yellow
  - Mate: Magenta
  - Divide: Cyan
  - Turn CW: Light blue
  - Turn CCW: Light red
  - Defend: Orange

#### GraphRenderer
Separate component for time-series graphs (1840×80):
```rust
pub struct GraphRenderer {
    width: usize,        // RENDER_WIDTH (1840)
    height: usize,       // BOTTOMBAR (80)
    view_offset: usize,  // Scroll position
    view_width: usize,   // Visible window (1840 ticks)
}
```

**Graph Metrics** (9 total):
1. Population count (white background bars)
2. Average genes per bug (gray line)
3. Average food per cell (green line)
4. Average bug mass (blue line)
5. Movement events (bright green line)
6. Starvation deaths (dark green line)
7. Drowning deaths (purple line)
8. Collision deaths (red line)
9. Births (magenta line)

**Auto-ranging**: Each visible window calculates min/max for optimal scaling

### bugs-cli
**Purpose**: Headless simulation runner

**Features**:
- Runs at maximum speed (no rendering delay)
- Progress bar with ETA
- Optional event recording
- Configurable tick count and seed

**Usage**:
```bash
bugs-cli --seed 42 --max-ticks 100000 --output run_001
```

### bugs-viewer-native
**Purpose**: Native GUI with real-time simulation

**Technology Stack**:
- `wgpu`: GPU rendering
- `egui`: Immediate mode GUI
- `winit`: Window management

**Features**:
- Live simulation view
- Playback speed control (1x, 10x, 100x)
- Mode switching (Bug Map / Environment)
- GPU-accelerated texture updates

### bugs-viewer-web
**Purpose**: WebAssembly viewer for browser replay

**Architecture**:
```rust
pub struct WebViewer {
    visualizer: Visualizer,           // Main world renderer
    graph_renderer: GraphRenderer,    // Graph renderer
    world: Option<World>,             // Current world state
    stats_history: VecDeque<WorldStats>,  // Rolling history (1300 ticks)
    pixel_buffer: Vec<u8>,            // 1840×1000×4 RGBA
    graph_buffer: Vec<u8>,            // 1840×80×4 RGBA
    main_canvas: HtmlCanvasElement,   // Main view canvas
    graph_canvas: HtmlCanvasElement,  // Graph canvas
    main_ctx: CanvasRenderingContext2d,
    graph_ctx: CanvasRenderingContext2d,
}
```

**JavaScript API**:
```javascript
// Constructor
new WebViewer(mainCanvasId, graphCanvasId)

// View control
.set_mode(isBugMap: bool)

// Data loading
.set_world(worldData: Uint8Array)  // Bincode serialized
.add_stats(statsData: Uint8Array)  // Bincode serialized

// Rendering
.render()  // Updates both canvases

// Graph navigation
.scroll_graph(delta: number)       // Relative scroll
.set_graph_offset(offset: number)  // Absolute position
.get_graph_offset(): number
.get_stats_count(): number

// Statistics
.get_stats(): string  // JSON format
```

**Update Loop**:
- Rendering: 20 FPS (50ms interval)
- Stats updates: 10 Hz (100ms interval)
- Graph scrolling: Immediate response

## Data Flow

### Simulation Run
```
bugs-cli
  └─> Simulation::new(seed)
      └─> World::new()
      └─> EventWriter::new() [optional]
  └─> Loop: tick()
      └─> Simulation::tick()
          └─> Update all bugs
          └─> Record events
          └─> Snapshot every N ticks
```

### Replay
```
bugs-viewer-web
  └─> WebViewer::new()
  └─> Load world data (bincode)
      └─> deserialize::<World>()
  └─> Load stats data (bincode)
      └─> deserialize::<WorldStats>()
  └─> render()
      └─> Visualizer::render_to_rgba()
          └─> Draw LEFTBAR
          └─> Draw world
      └─> GraphRenderer::render_to_rgba()
          └─> Calculate visible range
          └─> Draw 9 time-series
      └─> Update canvases via putImageData
```

## Performance Characteristics

### Native Simulation
- **Ticks/second**: 5,000-10,000 (depends on population)
- **Memory**: ~500KB for world + ~100KB per 1000 bugs
- **CPU**: Single-threaded (bug evaluation could be parallelized)

### WebAssembly
- **Ticks/second**: 500-2,000 (2-5x slower than native)
- **Memory**: Browser-limited (2-4GB typical)
- **Rendering**: Canvas 2D (~20 FPS for 1840×1080)

### Recording Overhead
- **Events**: ~2-5% slowdown
- **Snapshots**: ~10-20ms spike every N ticks
- **Disk usage**: ~1-5 MB per 10,000 ticks (compressed)

## Determinism Guarantees

The simulation is **fully deterministic**:

1. **Seedable RNG**: ChaCha8Rng with explicit seed
2. **Sorted iteration**: Bugs processed in ID order
3. **No floating-point non-determinism**: All physics is integer
4. **No external state**: Pure function of current state + RNG

**Verification**:
```bash
cargo test -p bugs-core -- test_determinism
```

## Extensibility Points

### Adding New Bug Actions
1. Add constant to `constants.rs`: `pub const ACT_NEW: usize = 8;`
2. Update `N_ACTIONS` constant
3. Implement in `Simulation::execute_action()`
4. Add color in `Visualizer::action_color()`

### Adding New Decision Types
1. Add to `DT_*` constants in `constants.rs`
2. Update `N_DECISIONS`
3. Implement in `BugBrain::decide_*()`

### Adding New Graph Metrics
1. Add field to `WorldStats` in `world.rs`
2. Track in `Simulation::tick()`
3. Add rendering in `GraphRenderer::render_to_rgba()`
4. Update web UI legend

### Custom Visualization Modes
1. Add variant to `VisMode` enum
2. Implement rendering in `Visualizer`
3. Add UI controls in viewers

## Build Targets

### Native (all platforms)
```bash
cargo build --release -p bugs-cli
cargo build --release -p bugs-viewer-native
```

### WebAssembly
```bash
cd crates/bugs-viewer-web
wasm-pack build --target web
```

**Output**: `pkg/` directory with `.js` and `.wasm` files

## Testing Strategy

### Unit Tests
- `bugs-core`: Genetic operations, world mechanics
- `bugs-recorder`: Serialization round-trips
- `bugs-render`: Pixel buffer dimensions

### Integration Tests
- Full simulation runs (determinism)
- Event recording and playback
- Snapshot compression/decompression

### Performance Tests
```bash
cargo bench -p bugs-core
```

## Debugging Tips

### View Internal State
```rust
// In simulation code
eprintln!("Bug {}: {:?}", bug.id, bug.brain);
```

### Trace Events
```bash
RUST_LOG=debug cargo run -p bugs-cli
```

### Inspect Recordings
```rust
use bugs_recorder::EventReader;
let reader = EventReader::new("recording.events")?;
for event in reader {
    println!("{:?}", event);
}
```

### Web Console
```javascript
// In browser console
viewer.get_stats()  // Current world statistics
viewer.get_stats_count()  // History length
```

## Common Issues

### Web Viewer Not Loading
- Ensure `wasm-pack build` completed successfully
- Check browser console for CORS errors
- Verify HTTP server is serving `.wasm` with correct MIME type

### Performance Degradation
- High population causes slowdown (quadratic collision checks)
- Reduce `--max-ticks` or increase snapshot interval
- Use `--no-record` for maximum speed

### Determinism Breaks
- Ensure no external input during simulation
- Check for uninitialized memory
- Verify RNG is seeded consistently

## References

- Original bugs.c: `/home/user/bugs/bugs.c`
- Constants reference: `crates/bugs-core/src/constants.rs`
- World format: `crates/bugs-core/src/world.rs`
- Rendering: `crates/bugs-render/src/lib.rs`
