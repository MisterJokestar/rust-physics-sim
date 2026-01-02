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

const TITLE: &str = "Plinko in Rust";
const WINDOW_WIDTH: u32 = 520;
const WINDOW_HEIGHT: u32 = 800;
const BACKGROUND: Color = Color::BLACK;
const COLLISION_LOOPS: u32 = 20;
const FONT_PATH: &str = "/usr/share/fonts/truetype/futuristic-font/Futuristic-MRer.ttf";

// Plinko
const BOXSIZE: u32 = 40;

fn main_loop(grid: &mut Grid, boxes: &mut Vec<i32>, canvas:&mut Canvas<Window>, font: &Font, dt: f32) {
    canvas.set_draw_color(BACKGROUND);
    canvas.clear();

    grid.draw_frame(canvas, dt);
    grid.update_boxes(canvas, boxes, font, BOXSIZE, WINDOW_HEIGHT);
    grid.cleanup();
    for _ in 0..COLLISION_LOOPS {
        grid.handle_collisions()
    }
}

fn set_up(grid: &mut Grid, boxes: &mut Vec<i32>) {
    grid.add_wall(Wall::new([0.0, 0.0], [0.0, WINDOW_HEIGHT as f32], Some(20), None, None, None));
    grid.add_wall(Wall::new([WINDOW_WIDTH as f32, 0.0], [WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32], Some(20), None, None, None));

    let num_areas = WINDOW_WIDTH / BOXSIZE;
    let num_plinkies = WINDOW_HEIGHT / 100 - 2;
    let plinkies_offset: u32 = 160;
    let plinkies_length: u32 = 20;
    for i in 1..num_areas {
        boxes.push(0);
        let x = i * BOXSIZE;
        grid.add_wall(Wall::new([x as f32, WINDOW_HEIGHT as f32 - 60.0], [x as f32, WINDOW_HEIGHT as f32 + 40.0], None, None, None, None));

        if i % 2 == 0 && i != num_areas - 1 {
            for j in (0..num_plinkies).step_by(2) {
                let y = j * 100;
                let color = Color::BLUE;
                add_plinky(grid, x, y, plinkies_offset, plinkies_length, color);
            }
        } else if i != 1 && i != num_areas - 1 {
            for j in (1..num_plinkies).step_by(2) {
                let y = j * 100;
                let color = Color::CYAN;
                add_plinky(grid, x, y, plinkies_offset, plinkies_length, color);
            }
        }
    }
    boxes.push(0)
}

fn add_plinky(grid: &mut Grid, x: u32, y: u32, offset: u32, length: u32, color: Color) {
    grid.add_wall(Wall::new(
        [x as f32, (y + offset) as f32], [(x + length) as f32, (y + length + offset) as f32],
        None, Some(color), None, None));
    grid.add_wall(Wall::new(
        [x as f32, (y + offset) as f32], [(x - length) as f32, (y + length + offset) as f32],
        None, Some(color), None, None));
}

fn spawn_balls(grid: &mut Grid) {
    let x: f32 = rand::random_range(20.0..(WINDOW_WIDTH as f32 - 20.0));
    let v: f32 = rand::random_range(-200.0..200.0);
    grid.add_ball(Ball::new([x, 60.0], Some([v, 0.0]), None, Some(Color::RED), None, None));
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window(TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let font = ttf_context.load_font(FONT_PATH, 24).unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(BACKGROUND);
    canvas.clear();
    canvas.present();
    let mut grid: Grid = Grid::new(50, 50, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
    let mut boxes: Vec<i32> = Vec::new();
    set_up(&mut grid, &mut boxes);

    let mut last_frame_time = Instant::now();
    let mut time: f32 = 0.0;
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event{
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                },
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;
        time += dt;

        if time > 1.2 {
            time = 0.0;
            spawn_balls(&mut grid);
        }

        main_loop(&mut grid, &mut boxes, &mut canvas, &font, dt);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
