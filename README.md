# rphys - A Lightweight 2D Physics Simulator

A minimalist 2D physics engine written in Rust, designed to demonstrate fundamental physics concepts with minimal external dependencies. This project showcases collision detection, spatial partitioning, and real-time physics simulation through an interactive demonstration.

## Overview

This physics simulator implements a complete 2D rigid body physics system from scratch, including:

- **Custom vector mathematics** - All vector operations (dot product, magnitude, normalization, etc.) are implemented without relying on external math libraries
- **Spatial partitioning grid** - Efficient collision detection using a uniform grid to reduce the number of collision checks from O(n²) to approximately O(n)
- **Impulse-based collision resolution** - Realistic collision responses with support for friction and restitution (bounciness)
- **Real-time simulation** - 60 FPS rendering with delta-time physics updates

The current demonstration simulates balls falling through a pegboard with gravity, bouncing off walls and pegs, and colliding with each other.

## Features

### Physics Engine
- ✨ **2D rigid body dynamics** with position, velocity, and force integration
- 🎯 **Collision detection** between circles (balls) and line segments (walls)
- 🔄 **Impulse-based collision resolution** with configurable friction and restitution
- 🌍 **Gravity** and other force applications
- 📊 **Spatial partitioning** for efficient broad-phase collision detection

### Technical Highlights
- 🔧 **Minimal dependencies** - Custom vector math library to avoid heavy mathematical dependencies
- ⚡ **Performance optimized** - Grid-based spatial partitioning reduces collision checks
- 🎨 **Real-time visualization** using SDL2 for graphics and rendering
- 📐 **Geometric collision handling** - Supports both line segment and endpoint (corner) collisions

## Dependencies

This project intentionally minimizes external dependencies. The only required dependencies are:

- `sdl2` - For window creation, rendering, and event handling
- `sdl2-gfx` - For drawing primitives (circles, lines)
- `sdl2-ttf` - For text rendering
- `rand` - For random number generation

**Note:** All vector mathematics and physics calculations are implemented from scratch in the `library` module to demonstrate the underlying concepts without relying on external physics or math libraries.

## Building and Running

### Prerequisites

1. **Rust toolchain** - Install from [rustup.rs](https://rustup.rs/)
2. **SDL2 development libraries** - Required for graphics
   - **Ubuntu/Debian**: `sudo apt-get install libsdl2-dev libsdl2-gfx-dev libsdl2-ttf-dev`
   - **macOS**: `brew install sdl2 sdl2_gfx sdl2_ttf`
   - **Windows**: Follow the [SDL2 setup guide](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc)

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd rphys

# Build and run in release mode (recommended for performance)
cargo run --release

# Or build in debug mode
cargo build
cargo run
```

### Controls

- **ESC** - Exit the simulation
- The simulation spawns balls automatically at regular intervals

## Project Structure

```
rphys/
├── src/
│   ├── main.rs      # Application entry point and simulation setup
│   ├── lib.rs       # Custom vector mathematics library
│   ├── items.rs     # Physics objects (Ball, Wall) and collision logic
│   └── grid.rs      # Spatial partitioning grid for collision optimization
└── Cargo.toml       # Project dependencies and configuration
```

### Module Breakdown

#### `lib.rs` - Vector Mathematics Library
Custom implementations of 2D vector operations:
- Dot product calculation
- Vector magnitude (length)
- Vector normalization
- Vector arithmetic (finding vectors between points)
- Normal vector computation (perpendicular vectors)

#### `items.rs` - Physics Objects
Defines the core physics entities:
- **Ball**: Dynamic circular objects with velocity, position, and collision response
- **Wall**: Static line segments that balls can collide with
- **Collision methods**: Detailed impulse-based collision resolution with friction and restitution

#### `grid.rs` - Spatial Partitioning
Efficient collision detection system:
- Divides the simulation space into a uniform grid of cells
- Tracks which objects are in which cells
- Only checks collisions between objects in the same or adjacent cells
- Includes a DDA-like line traversal algorithm for walls spanning multiple cells

#### `main.rs` - Application Entry Point
Sets up the simulation demonstration:
- Initializes SDL2 and creates the rendering window
- Constructs the pegboard layout with walls and pegs
- Runs the main game loop at 60 FPS
- Handles ball spawning and cleanup

## How It Works

### Physics Simulation Loop

Each frame, the simulator performs the following steps:

1. **Update velocities** - Apply forces (gravity) to all balls
2. **Update positions** - Move balls based on their velocities
3. **Detect collisions** - Use the spatial grid to find nearby objects
4. **Resolve collisions** - Apply impulse-based collision resolution (repeated 20 times per frame for stability)
5. **Render** - Draw all objects to the screen
6. **Cleanup** - Remove balls that have left the simulation area

### Collision Detection

The spatial partitioning grid divides the world into cells (50x50 pixels by default). Each physics object is registered in the cells it occupies:

- **Balls**: Registered in the cell containing their center point
- **Walls**: Registered in all cells that the line segment passes through (using a line traversal algorithm)

When checking for collisions, each ball only examines objects in its own cell and the 8 adjacent cells (a 3×3 grid).

### Collision Resolution

Collisions use impulse-based resolution:

1. **Detect penetration** - Calculate if and how much objects are overlapping
2. **Decompose velocity** - Split velocity into normal (perpendicular) and tangent (parallel) components relative to the collision surface
3. **Apply restitution** - Reverse and scale the normal velocity based on the restitution coefficient (bounciness)
4. **Apply friction** - Reduce the tangent velocity based on the friction coefficient
5. **Resolve penetration** - Push objects apart to eliminate overlap

## Configuration

Key constants can be adjusted in `src/main.rs`:

- `WINDOW_WIDTH` / `WINDOW_HEIGHT` - Simulation window size
- `COLLISION_LOOPS` - Number of collision resolution iterations per frame (higher = more stable but slower)
- Grid cell size (in `Grid::new()` call) - Affects collision detection performance

Physics properties can be adjusted when creating objects:
- `friction` - How much tangential velocity is lost in collisions (0.0 = frictionless, 1.0 = maximum friction)
- `restitution` - How much normal velocity is preserved in collisions (0.0 = no bounce, 1.0 = perfectly elastic)

## Performance Characteristics

- **Without spatial partitioning**: O(n²) collision checks for n objects
- **With spatial partitioning**: Approximately O(n) collision checks (assuming even distribution)
- **Typical performance**: Handles hundreds of balls at 60 FPS on modern hardware

## Future Enhancements

Potential areas for expansion:
- [ ] Add rotational dynamics (angular velocity, torque)
- [ ] Implement more shape types (polygons, rectangles)
- [ ] Add constraints and joints (distance constraints, hinges)
- [ ] Implement a more sophisticated broadphase (quadtree, BVH)
- [ ] Add configurable simulation parameters via UI or config file
- [ ] Support for different force fields (wind, magnetism)

## License

This project is provided as-is for educational and demonstration purposes.

## Acknowledgments

This project was built to demonstrate fundamental physics simulation concepts with minimal dependencies, showing how spatial partitioning and impulse-based collision resolution work under the hood.
