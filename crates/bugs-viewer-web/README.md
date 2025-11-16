# Bugs Web Viewer

WebAssembly-based replay viewer for Bugs 0.29 evolution simulations.

## Features

- **Dual Canvas System**:
  - Main canvas (1840×1000): World view with LEFTBAR activity visualization
  - Graph canvas (1840×80): Scrollable time-series graphs

- **Interactive Controls**:
  - View mode toggle (Bug Map / Environment)
  - Graph scrolling (navigate through 1300 ticks of history)
  - Real-time statistics display

- **Visualization**:
  - LEFTBAR (80px): Row-by-row activity distribution with color-coded action bars
  - Main world: Bug ethnicity trails or environment (food/water)
  - Time-series graphs: 9 metrics including population, genes, deaths, births

## Building

Install wasm-pack if you haven't already:

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

Build the WebAssembly module:

```bash
cd crates/bugs-viewer-web
wasm-pack build --target web
```

This creates a `pkg/` directory with:
- `bugs_viewer_web.js` - JavaScript bindings
- `bugs_viewer_web_bg.wasm` - WebAssembly module

## Running

Serve the directory with any HTTP server:

```bash
# Using Python
python3 -m http.server 8000

# Using Node.js
npx http-server -p 8000

# Using Rust
cargo install simple-http-server
simple-http-server -p 8000
```

Open http://localhost:8000 in your browser.

## API

The `WebViewer` class provides:

```javascript
// Constructor
const viewer = new WebViewer('mainCanvasId', 'graphCanvasId');

// View mode
viewer.set_mode(true);  // Bug map
viewer.set_mode(false); // Environment map

// World state
viewer.set_world(worldData);    // Load world (bincode serialized)
viewer.add_stats(statsData);    // Add tick stats (bincode serialized)

// Rendering
viewer.render();  // Render both canvases

// Graph controls
viewer.scroll_graph(-100);      // Scroll back 100 ticks
viewer.set_graph_offset(0);     // Jump to live view
viewer.get_graph_offset();      // Get current offset
viewer.get_stats_count();       // Get history length

// Statistics
viewer.get_stats();  // Get current world stats as JSON
```

## File Format

The viewer expects bincode-serialized Rust data:

- **World data**: `bincode::serialize(&world)`
- **Stats data**: `bincode::serialize(&stats)`

This allows efficient sharing of simulation recordings.

## Performance

- Rendering: 20 FPS (50ms interval)
- Stats updates: 10 Hz (100ms interval)
- History buffer: 1300 ticks (configurable)
- Canvas 2D rendering (no WebGL required)

## Browser Compatibility

Tested on:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

Requires WebAssembly support (all modern browsers).
