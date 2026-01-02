# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

The Kessler Syndrome Simulator is a high-performance web-based 3D simulation of the cascading collision of space debris. Built with Rust and Bevy 0.16.1, compiled to WebAssembly, it simulates how satellite collisions could create a chain reaction that renders Earth's orbital environment unusable.

**Core Technologies**: Rust, Bevy 0.16.1 ECS, WebAssembly, WebGPU, SGP4 orbital mechanics

## Essential Development Commands

### Building and Running

```bash
# Prerequisites
# - Install Rust (latest stable)
# - Install wasm-pack: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM package for web
wasm-pack build --target web --out-dir pkg

# Or use the build script (also used by Netlify)
./build.sh

# Serve locally (from project root)
python -m http.server 8000
# Then open http://localhost:8000
```

### Testing and Validation

The project includes a built-in stress testing framework accessed via keyboard controls:
- **Press 'B'**: Spawn stress test satellites (up to 5000)
- **Press 'Shift+B'**: Clear all stress test objects
- Performance metrics are logged to the browser console

### Deployment

- **Netlify**: Push to git repository connected to Netlify - `netlify.toml` handles automatic deployment using `build.sh`
- **Manual**: The `build.sh` script creates a `dist/` directory with all necessary files

## Architecture Overview

### Bevy Entity Component System (ECS)

The entire simulation is built on Bevy's ECS architecture:

**Components** (data attached to entities):
- `OrbitalState` - position, velocity, mass (km and kg)
- `Satellite` / `Debris` - object classification
- `PhysicsObject` - collision radius, drag coefficient
- `TleData` - Two-Line Element orbital data reference
- `OptimizedPhysics` - marker for SIMD-optimized objects
- `StressTestObject` - marker for performance test objects

**Resources** (global state):
- `Constants` - Earth mass, gravitational parameter (GM), Earth radius
- `SimulationTime` - current simulation time, speed multiplier (1x to 86400x), paused state
- `EnergyAnalytics` - altitude bins, energy tracking, object counts
- `TleDataCache` - cached TLE records
- `SpatialOctree` - collision detection spatial partitioning
- `OptimizedPhysicsData` - SIMD-aligned physics arrays
- `StressTestConfig` - stress test parameters

### Dual Physics Architecture

The simulator maintains two physics systems:

1. **Standard Physics** (`systems/physics.rs`): Traditional ECS-based orbital mechanics
2. **Optimized Physics** (`systems/optimized_physics.rs`): SIMD-optimized parallel processing using Rayon

Both systems run in parallel by default. The optimized system uses 32-byte aligned data structures for vectorization and processes all physics calculations in parallel.

### Physics Model

- **Force**: F = -GMm/r² × r̂
- **Acceleration**: a = -GM × r / |r|³
- **Gravitational Parameter**: μ = GM = 3.986004418×10¹⁴ m³/s²
- **Units**: km for distance, km/s for velocity, kg for mass
- **Integration**: Euler method with configurable timestep

### Data Pipeline

```
TLE Data (assets/tles/*.tle or Celestrak API)
  → parse_tle_data() (systems/data.rs)
  → tle_to_state_vectors() using SGP4 (utils/sgp4_wrapper.rs)
  → spawn_satellites_from_records()
  → ECS entities with OrbitalState + Satellite
  → physics_system() or optimized_physics_system()
  → collision_detection_system() with octree
  → debris_generation_system() using NASA breakup model
  → rendering systems
```

### Collision Detection

Multi-phase collision detection:
1. **Broad Phase**: Octree spatial partitioning (covers ±50,000km, 6 levels deep)
2. **Narrow Phase**: Sphere-sphere distance testing
3. **Response**: NASA Standard Breakup Model generates 2-50 debris pieces based on collision energy

### System Organization (lib.rs)

Systems are organized into parallel execution groups:
- **Startup**: Scene setup, TLE data initialization
- **Update - Input**: Camera controls, time control, satellite selection
- **Update - Physics**: Standard physics, optimized physics (3 systems), physics monitoring
- **Update - Collision**: Octree rebuild, collision detection, debris generation
- **Update - Rendering**: Satellite/debris visualization, collision flashes, position updates
- **Update - Analytics**: Energy tracking, debugging, TLE fetch processing
- **Update - Stress Test**: Satellite spawning, cleanup, performance monitoring
- **Update - Debris Mechanics**: Random debris injection, orbital decay
- **Update - UI**: Satellite selection, info display

### File Structure

```
src/
├── lib.rs                 # WASM entry point, app initialization, scene setup
├── components/            # ECS component definitions
│   ├── orbital.rs        # OrbitalState, TleData
│   ├── objects.rs        # Satellite, Debris, CollisionFlash, render markers
│   └── physics.rs        # PhysicsObject collision properties
├── systems/              # Game logic systems
│   ├── physics.rs        # Standard orbital mechanics (Euler integration)
│   ├── optimized_physics.rs  # SIMD-parallel physics with Rayon
│   ├── collision.rs      # Octree spatial partitioning, collision detection
│   ├── analytics.rs      # Energy tracking, statistics
│   ├── rendering.rs      # 3D visualization, camera controls
│   ├── data.rs           # TLE fetching, parsing, satellite spawning
│   ├── stress_test.rs    # Performance testing framework
│   └── debris_mechanics.rs # Orbital decay, random debris injection
├── resources/            # Global state
│   ├── constants.rs      # Physical constants (GM, Earth mass/radius)
│   └── simulation.rs     # SimulationTime, EnergyAnalytics, configs
└── utils/                # Utility functions
    ├── tle_parser.rs     # Two-Line Element format parser
    └── sgp4_wrapper.rs   # SGP4 orbital mechanics library wrapper
```

## Key Implementation Details

### Coordinate System
- **Units**: km for distances, km/s for velocities (Earth radius = 6371 km = 6.371 units)
- **Origin**: Earth center at (0, 0, 0)
- **Scale**: Direct 1:1 mapping (no unit conversion needed during physics)

### Time Control
- **Simulation Time**: Tracked in `SimulationTime` resource
- **Speed Multiplier**: 1× (real-time) to 86,400× (1 second = 1 day)
- **Timestep**: Configurable physics timestep (default optimized for stability)

### TLE Data Loading
1. **Local files**: Checks `assets/tles/*.tle` first
2. **Celestrak API**: Falls back to `https://celestrak.org/NORAD/elements/gp.php?GROUP=active&FORMAT=tle`
3. **Test data**: Generates test satellites if no data available

### Rendering Conventions
- **Satellites**: Green spheres (marker: `RenderAsSatellite`)
- **Debris**: Red spheres (marker: `RenderAsDebris`)
- **Collision flashes**: Yellow/white transient markers
- **Earth**: Textured sphere with bathymetry at origin

## Bevy 0.16.1 Specific Patterns

The codebase has been updated for Bevy 0.16.1 breaking changes:
- `Input<T>` → `ButtonInput<T>` for all input systems
- `shape::UVSphere` → `primitives::Sphere` (and `.mesh().uv()` for mesh creation)
- `delta_seconds()` → `delta_secs()`, `elapsed_seconds()` → `elapsed_secs()`
- `KeyCode::Key1` → `KeyCode::Digit1` (and similar for other digits)
- Colors use `Color::srgb()` instead of `Color::rgb()`
- Component insertion: Use `.insert()` when building entities (not bundles)

## Performance Considerations

- **Stress Testing**: Tested up to 5000 satellites with performance monitoring
- **Optimization**: SIMD alignment, parallel processing, octree spatial partitioning
- **Real-world Data**: Successfully loads 12,148+ real satellite TLE records
- **Benchmarks**: ~60 FPS at 100 satellites, ~10 FPS at 5000 satellites

## Controls Reference

- **Mouse Drag**: Rotate camera around Earth
- **Mouse Wheel**: Zoom in/out
- **Space**: Pause/Resume simulation
- **1-4**: Set time speed (1x, 60x, 3600x, 86400x)
- **B**: Spawn stress test satellites
- **Shift+B**: Clear stress test objects
- **Click**: Select satellite to view orbital information
