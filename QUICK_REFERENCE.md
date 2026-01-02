# Kessler Simulator - Quick Reference

## Run the Simulator
```bash
cargo run --release
```

## Key Controls

### Simulation Control
- **Space** - Pause/Resume
- **1** - 1x speed (real-time)
- **2** - 60x speed (1 second = 1 minute)
- **3** - 3600x speed (1 second = 1 hour)
- **4** - 86400x speed (1 second = 1 day)

### Object Spawning
- **5** - Spawn 500 objects
- **6** - Spawn 1,000 objects
- **7** - Spawn 2,000 objects
- **8** - Spawn 5,000 objects
- **9** - Spawn 100,000 objects (smooth spawn over time)
- **0** - Spawn 1,000,000 objects (smooth spawn over time)
- **B** - Spawn stress test objects (based on current settings)
- **C** - Clear all stress test objects

### Render Mode
- **M** - Toggle render density mode:
  - FULL: All objects rendered (default, <10k)
  - HIGH DENSITY: 30% rendered (10k-50k)
  - EXTREME DENSITY: 10% rendered (100k+)

### Camera
- **Mouse Drag** - Rotate around Earth
- **Mouse Scroll** - Zoom in/out

## Console HUD Output (Updates Every Second)

```
========================================
FPS: 20.6 (48.46 ms)
Objects: 11165 total (11000 satellites, 165 debris)
Collisions this frame: 62
Sim Speed: 3600x | Sim Time: 3487.95s
Status: RUNNING
Spawning: 10000/10000
========================================
```

### What These Mean:
- **FPS**: Frames per second (higher is better, 60+ is excellent, <30 is concerning)
- **Objects**: Total simulation objects with breakdown
- **Collisions**: How many collision pairs detected this frame
- **Sim Speed**: Current time multiplier
- **Sim Time**: Elapsed simulation time in seconds
- **Status**: RUNNING (green) or PAUSED (red)
- **Spawning**: Progress if spawning stress test objects

## Performance Benchmarks (Approximate)

| Objects | Expected FPS | Notes |
|---------|--------------|-------|
| 500     | 60+           | Excellent |
| 1,000   | 60+           | Excellent |
| 5,000   | 45-60         | Good |
| 10,000  | 25-30         | Acceptable |
| 50,000  | 8-12          | Use HIGH DENSITY mode |
| 100,000 | 3-8           | Use EXTREME DENSITY mode |
| 1,000,000 | 1-3          | Use EXTREME DENSITY mode, expect slow |

## Optimization Implemented

### Phase 1: Quick Wins ✓
- Console HUD showing real-time statistics
- Smooth spawning (1000-5000 objects/frame instead of 10,000-200,000)
- Shared material caching (no per-object allocations)

### Phase 2: Render Control ✓
- Density modes to limit rendered objects at scale
- Automatic mode switching based on object count

### Phase 3: Spatial/Physics ✓
- Incremental octree updates (only moved objects)
- Optimized collision detection with pre-allocation
- Frame skipping at high object counts
- Adaptive physics timestep for high speeds

## Troubleshooting

### Low FPS (<10)
1. Press **M** to switch to HIGH or EXTREME density mode
2. Press **C** to clear some objects
3. Pause with **Space** if simulation is overwhelming
4. Check collision rate - if high, satellites are colliding too frequently

### Simulation Too Fast
1. Press **1** or **2** to reduce time speed
2. Press **Space** to pause

### Too Few Objects
1. Press **5-8** to add more objects
2. Press **B** to spawn stress test batch
3. Press **9** or **0** for massive object counts (smooth spawn)

### Spawning Not Completing
1. Check console "Spawning: X/Y" to see progress
2. If stuck, press **C** to clear and start over
3. Spawning rate is 1000-5000 objects/frame, so 10k takes ~2-10 seconds

## Building

### Development Build (Faster, slower runtime)
```bash
cargo run
```

### Release Build (Slower, optimized runtime)
```bash
cargo run --release
```

## Architecture

### Core Systems
- **Physics**: Standard ECS + Optimized SIMD parallel
- **Collision**: Octree spatial partitioning
- **Rendering**: Bevy 3D with material caching
- **Data**: TLE loading and satellite spawning
- **Analytics**: Energy tracking, collision counting

### Key Components
- `OrbitalState`: Position, velocity, mass
- `PhysicsObject`: Collision radius, drag coefficient
- `Satellite`/`Debris`: Object classification
- `PreviousPosition`: Tracks movement for octree updates

## File Locations

### Source Code
- `src/main.rs` - Application entry
- `src/systems/` - All simulation systems
- `src/components/` - ECS components
- `src/resources/` - Global state
- `src/utils/` - Helper functions

### Documentation
- `OPTIMIZATION_SESSION.md` - Detailed optimization log
- `QUICK_REFERENCE.md` - This file
- `CLAUDE.md` - Original project documentation

## Next Steps (Future Work)

1. **GPU Instancing**: Custom shaders for 10x+ performance boost
2. **Headless Mode**: Record simulation without rendering
3. **Playback System**: Replay recorded simulations
4. **Timeline UI**: Scrub through simulation time
