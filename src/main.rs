//! A Plinko physics simulation game written in Rust.
//!
//! This application simulates a Plinko board where balls fall through a grid of pegs
//! and collect in boxes at the bottom. It uses a custom 2D physics engine with
//! spatial partitioning for efficient collision detection.

mod items;
mod grid;

use crate::items::{Ball, Wall};
use crate::grid::Grid;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas};
use sdl2::ttf::Font;
use rand;
use std::time::{Duration, Instant};

/// Window title displayed in the title bar
const TITLE: &str = "Plinko in Rust";
/// Width of the simulation window in pixels
const WINDOW_WIDTH: u32 = 520;
/// Height of the simulation window in pixels
const WINDOW_HEIGHT: u32 = 800;
/// Background color for the canvas
const BACKGROUND: Color = Color::BLACK;
/// Number of collision resolution iterations per frame
const COLLISION_LOOPS: u32 = 20;
/// Path to the font file used for rendering text
const FONT_PATH: &str = "/usr/share/fonts/truetype/futuristic-font/Futuristic-MRer.ttf";
/// Width of each collection box at the bottom in pixels
const BOXSIZE: u32 = 40;

/// Main game loop that updates and renders the simulation for one frame.
///
/// # Arguments
///
/// * `grid` - The spatial grid containing all physics objects
/// * `boxes` - Vector tracking ball counts for each collection box
/// * `canvas` - SDL2 canvas for rendering
/// * `font` - Font for rendering text
/// * `dt` - Time delta in seconds since last frame
fn main_loop(grid: &mut Grid, boxes: &mut Vec<i32>, canvas:&mut Canvas<Window>, font: &Font, dt: f32) {
    canvas.set_draw_color(BACKGROUND);
    canvas.clear();

    grid.draw_frame(canvas, dt);
    grid.update_boxes(canvas, boxes, font, BOXSIZE, WINDOW_HEIGHT);
    grid.cleanup();
    // Run multiple collision passes per frame for stability
    for _ in 0..COLLISION_LOOPS {
        grid.handle_collisions()
    }
}

/// Sets up the Plinko board with walls, pegs, and collection boxes.
///
/// Creates the border walls, collection box dividers, and arranges the pegs
/// in a staggered pattern across the board.
///
/// # Arguments
///
/// * `grid` - The spatial grid to add objects to
/// * `boxes` - Vector to initialize for tracking ball counts
fn set_up(grid: &mut Grid, boxes: &mut Vec<i32>) {
    // Add left and right border walls
    grid.add_wall(Wall::new([0.0, 0.0], [0.0, WINDOW_HEIGHT as f32], Some(20), None, None, None));
    grid.add_wall(Wall::new([WINDOW_WIDTH as f32, 0.0], [WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32], Some(20), None, None, None));

    // Calculate Plinko board dimensions
    let num_areas = WINDOW_WIDTH / BOXSIZE;
    let num_plinkies = WINDOW_HEIGHT / 100 - 2;
    let plinkies_offset: u32 = 160;
    let plinkies_length: u32 = 20;

    // Create collection boxes and pegs
    for i in 1..num_areas {
        boxes.push(0);
        let x = i * BOXSIZE;
        // Add divider wall for collection box
        grid.add_wall(Wall::new([x as f32, WINDOW_HEIGHT as f32 - 60.0], [x as f32, WINDOW_HEIGHT as f32 + 40.0], None, None, None, None));

        // Add pegs in staggered rows (alternating pattern for Plinko effect)
        if i % 2 == 0 && i != num_areas - 1 {
            // Even columns: pegs on even rows
            for j in (0..num_plinkies).step_by(2) {
                let y = j * 100;
                let color = Color::BLUE;
                add_plinky(grid, x, y, plinkies_offset, plinkies_length, color);
            }
        } else if i != 1 && i != num_areas - 1 {
            // Odd columns: pegs on odd rows
            for j in (1..num_plinkies).step_by(2) {
                let y = j * 100;
                let color = Color::CYAN;
                add_plinky(grid, x, y, plinkies_offset, plinkies_length, color);
            }
        }
    }
    boxes.push(0)
}

/// Adds a plinky (peg) to the grid as two diagonal walls forming a V shape.
///
/// # Arguments
///
/// * `grid` - The spatial grid to add the plinky to
/// * `x` - Horizontal position of the peg center
/// * `y` - Vertical position of the peg base
/// * `offset` - Vertical offset from y position
/// * `length` - Length of each diagonal line
/// * `color` - Color to render the peg
fn add_plinky(grid: &mut Grid, x: u32, y: u32, offset: u32, length: u32, color: Color) {
    // Right diagonal line (going down and right)
    grid.add_wall(Wall::new(
        [x as f32, (y + offset) as f32], [(x + length) as f32, (y + length + offset) as f32],
        None, Some(color), None, None));
    // Left diagonal line (going down and left)
    grid.add_wall(Wall::new(
        [x as f32, (y + offset) as f32], [(x - length) as f32, (y + length + offset) as f32],
        None, Some(color), None, None));
}

/// Spawns a new ball at a random horizontal position near the top.
///
/// # Arguments
///
/// * `grid` - The spatial grid to add the ball to
fn spawn_balls(grid: &mut Grid) {
    // Random horizontal position (avoiding edges)
    let x: f32 = rand::random_range(20.0..(WINDOW_WIDTH as f32 - 20.0));
    // Random initial horizontal velocity
    let v: f32 = rand::random_range(-200.0..200.0);
    grid.add_ball(Ball::new([x, 60.0], Some([v, 0.0]), None, Some(Color::RED), None, None));
}

/// Main entry point for the Plinko simulation.
///
/// Initializes SDL2, creates the window and rendering context, sets up the Plinko board,
/// and runs the main game loop at 60 FPS.
fn main() {
    // Initialize SDL2 subsystems
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    // Create window and font
    let window = video_subsystem.window(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let font = ttf_context.load_font(FONT_PATH, 24).unwrap();

    // Create rendering canvas
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(BACKGROUND);
    canvas.clear();
    canvas.present();

    // Initialize physics grid with 50x50 pixel cells
    let mut grid: Grid = Grid::new(50, 50, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
    let mut boxes: Vec<i32> = Vec::new();
    set_up(&mut grid, &mut boxes);

    // Timing variables
    let mut last_frame_time = Instant::now();
    let mut time: f32 = 0.0;
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Main game loop
    'running: loop {
        // Handle events (quit on Escape or window close)
        for event in event_pump.poll_iter() {
            match event{
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                _ => {}
            }
        }

        // Calculate delta time
        let now = Instant::now();
        let dt = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;
        time += dt;

        // Spawn a new ball every 1.2 seconds
        if time > 1.2 {
            time = 0.0;
            spawn_balls(&mut grid);
        }

        // Update and render the simulation
        main_loop(&mut grid, &mut boxes, &mut canvas, &font, dt);

        canvas.present();
        // Target 60 FPS
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
