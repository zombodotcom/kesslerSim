# Kessler Syndrome Simulator (Web)

A high-performance web-based 3D simulation of the Kessler syndrome - the cascading collision of space debris that could render Earth's orbital environment unusable. Built with Bevy (Rust) compiled to WebAssembly for maximum performance.

## Features

- **Real Satellite Data**: Loads real TLE (Two-Line Element) data from local files or test datasets
- **Advanced Physics**: Full orbital mechanics with SGP4 propagation
- **Collision Detection**: Octree-based spatial partitioning for efficient collision detection
- **NASA Breakup Model**: Realistic debris generation based on collision energy
- **Random Debris Injection**: Simulates ongoing launches and background space activity
- **Orbital Decay**: Atmospheric drag model removes low-altitude debris
- **Interactive Tracking**: Click on satellites to view orbital parameters
- **High Performance**: Optimized physics with SIMD and parallel processing

## Building

### Prerequisites

- Rust (latest stable)
- wasm-pack (`curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`)

### Build for Web

```bash
# Build WASM package
wasm-pack build --target web --out-dir pkg

# The output will be in the pkg/ directory
# Serve with any static file server, e.g.:
python -m http.server 8000
```

### Build for Netlify

The project includes a `netlify.toml` configuration. Simply push to a Git repository connected to Netlify, and it will build automatically using the `build.sh` script.

## Controls

- **Mouse Drag**: Rotate camera around Earth
- **Mouse Wheel**: Zoom in/out
- **Space**: Pause/Resume simulation
- **1-4**: Set time speed (1x, 60x, 3600x, 86400x)
- **Click**: Select satellite to view info

## Project Structure

```
.
├── src/
│   ├── lib.rs              # WASM entry point
│   ├── components/         # ECS components
│   ├── systems/           # Game systems (physics, collision, rendering)
│   ├── resources/          # Global resources
│   └── utils/              # Utilities (TLE parser, SGP4)
├── assets/
│   ├── tles/              # TLE data files
│   └── textures/          # Earth textures
├── index.html             # Web entry point
├── Cargo.toml            # Rust dependencies
└── netlify.toml          # Netlify configuration
```

## Technical Details

- **Engine**: Bevy 0.16.1
- **Language**: Rust
- **Target**: WebAssembly (WASM)
- **Physics**: 2-body orbital mechanics with SGP4 propagation
- **Collision**: Octree spatial partitioning
- **Rendering**: Bevy's 3D renderer with WebGPU

## References

This project is a Rust/WebAssembly port and enhancement of several reference implementations:

- **`reference/kessler/`** - Original Rust Bevy implementation with advanced physics, collision detection, and SGP4 propagation. This served as the primary codebase foundation.
- **`reference/Kessler_Simulations/`** - Python-based network simulation models for studying Kessler syndrome dynamics, phase transitions, and debris evolution. Concepts from this project (random debris injection, orbital decay) were ported to Rust.
- **`reference/OrbitSmash/`** - Original Three.js/JavaScript implementation that demonstrated the web-based visualization approach. This project inspired the web deployment strategy.
- **NASA Standard Breakup Model** - Used for realistic debris generation based on collision energy and mass.

### Acknowledgments

Special thanks to the original developers:
- The Bevy reference implementation team
- Xiaoxuan Zhang, Maarten Stork, Zoë Azra Blei, and Ilias-Panagiotis Sofianos (Kessler_Simulations)
- Jenna de Vries, Kato Schmidt, and Derk Niessink (OrbitSmash)

## Testing

The project includes comprehensive tests covering unit tests, integration tests, property-based tests, and WASM-specific tests.

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test suite
cargo test --test tle_parser_test
cargo test --test physics_test
cargo test --test integration_test

# Run property-based tests
cargo test --test property_energy_test
cargo test --test property_orbital_test
cargo test --test property_collision_test

# Run WASM tests (requires wasm-pack)
wasm-pack test --headless --firefox

# Run benchmarks
cargo bench
```

### Test Coverage

The test suite covers:

- **Unit Tests**: Core functionality for each system
  - TLE parsing and validation
  - SGP4 conversion and orbital mechanics
  - Physics calculations and energy conservation
  - Collision detection and spatial partitioning
  - Debris generation and NASA breakup model
  - Analytics and energy binning

- **Integration Tests**: System interactions
  - Full simulation cycles
  - Collision cascades (Kessler syndrome)
  - Data loading and satellite spawning
  - Physics and collision integration

- **Property-Based Tests**: Mathematical correctness
  - Energy conservation across random states
  - Orbital mechanics preservation
  - Collision detection correctness

- **WASM Tests**: Browser compatibility
  - WASM compilation and exports
  - Memory management
  - Cross-platform compatibility

### Test Fixtures

Test data is located in `tests/fixtures/`:
- `iss.tle` - Known ISS TLE for validation
- `hubble.tle` - Hubble Space Telescope TLE
- `test_satellites.tle` - Small test dataset
- `malformed.tle` - Edge case TLE data

### Success Criteria

- **Coverage**: >80% code coverage for core systems (physics, collision, TLE parsing)
- **Accuracy**: Physics tests match known orbital mechanics within 1% error
- **Performance**: WASM tests maintain 60 FPS with 100+ objects
- **Reliability**: All tests pass consistently in CI/CD

## License

MIT License

