use rphys::library::*;
use crate::items::{PhysItem, Ball, Wall, GRAVITY};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::ttf::Font;
use std::collections::HashMap;

pub struct Grid {
    unit_width: i32,
    unit_height: i32,
    grid: Vec<Vec<Section>>,
    out_of_bounds: Section,
    walls: HashMap<usize, Wall>,
    balls: HashMap<usize, Ball>,
    x_units: i32,
    y_units: i32,
    ball_cnt: usize,
    ball_id: usize,
    wall_cnt: usize,
    wall_id: usize,
}

pub struct Section {
    pub id: [usize; 2],
    pub items: Vec<PhysItem>,
}

impl Section {
    pub fn remove_ball(&mut self, id: usize) {
        self.items.retain( |item| {
            match item {
                PhysItem::Ball(ball_id) => return *ball_id != id,
                _ => return true
            }
        });
    }
}

impl Grid {
    pub fn new(unit_width: i32, unit_height: i32, window_width: i32, window_height: i32) -> Grid {
        let mut grid = Grid {
            unit_width: unit_width,
            unit_height: unit_height,
            grid: Vec::new(),
            out_of_bounds: Section {
                id: [usize::MAX, usize::MAX],
                items: Vec::new(),
            },
            walls: HashMap::new(),
            balls: HashMap::new(),
            x_units: (window_width + unit_width * 2) / unit_width,
            y_units: (window_height + unit_height * 2) / unit_height,
            ball_cnt: 0,
            ball_id: 0,
            wall_cnt: 0,
            wall_id: 0,
        };
        for i in 0..(grid.x_units as usize) {
            grid.grid.push(Vec::new());
            for j in 0..(grid.y_units as usize) {
                grid.grid[i].push(Section {
                    id: [i, j],
                    items: Vec::new(),
                });
            }
        }
        grid
    }

    pub fn get_section(&mut self, x: i32, y: i32) -> &mut Section {
        if x < 0 || x >= self.x_units || y < 0 || y >= self.y_units {
            return &mut self.out_of_bounds;
        }
        &mut self.grid[x as usize][y as usize]
    }

    pub fn get_section_at_position(&mut self, x: f32, y: f32) -> &mut Section {
        let x_unit = (x as i32 + self.unit_width) / self.unit_width;
        let y_unit = (y as i32 + self.unit_height) / self.unit_height;
        self.get_section(x_unit, y_unit)
    }

    pub fn get_sections_between_points(&mut self, s: [f32; 2], e: [f32; 2]) -> Vec<[usize; 2]> {
        let vec = normalize(find_vector(s, e));
        let vx = vec[0];
        let vy = vec[1];

        let mut curr_x_unit = (s[0] as i32 + self.unit_width) / self.unit_width;
        let mut curr_y_unit = (s[1] as i32 + self.unit_height) / self.unit_height;
        let mut relative_x = s[0] % self.unit_width as f32;
        let mut relative_y = s[1] % self.unit_height as f32;

        let [mut curr_x, mut curr_y] = self.get_section(curr_x_unit, curr_y_unit).id;
        let [end_x, end_y] = self.get_section_at_position(e[0], e[1]).id;

        if (vx == 0.0 && vy == 0.0) || (curr_x == end_x && curr_y == end_y) {
            return vec![[curr_x, curr_y]];
        }

        let mut visited_ids: Vec<[usize; 2]> = Vec::new();

        let max_steps = (self.x_units + self.y_units) * 2 + 10;
        let mut steps = 0;

        'get_sections: loop {
            if !visited_ids.contains(&[curr_x, curr_y]) {
                visited_ids.push([curr_x, curr_y]);
            }

            if curr_x == end_x && curr_y == end_y {
                break 'get_sections;
            }

            let mut tx: f32;
            if vx < 0.0 {
                tx = (-relative_x) / vx;
            } else if vx > 0.0 {
                tx = (self.unit_width as f32 - relative_x) / vx;
            } else {
                tx = f32::INFINITY;
            }
            let mut ty: f32;
            if vy < 0.0 {
                ty = (-relative_y) / vy;
            } else if vy > 0.0 {
                ty = (self.unit_height as f32 - relative_y) / vy;
            } else {
                ty = f32::INFINITY;
            }

            if tx <= 0.0 { tx = f32::INFINITY; }
            if ty <= 0.0 { ty = f32::INFINITY; }

            let t: f32;
            if tx < ty {
                t = tx;
                if vx > 0.0 { curr_x_unit += 1 } else { curr_x_unit -= 1 };

                relative_x = if vx > 0.0 { 0.0 } else { self.unit_width as f32 };
                relative_y = vy * t;
            } else {
                t = ty;
                if vy > 0.0 { curr_y_unit += 1 } else { curr_y_unit -= 1 };

                relative_x = vx * t;
                relative_y = if vy > 0.0 { 0.0 } else { self.unit_height as f32 };
            }

            if curr_x_unit < 0 || curr_x_unit >= self.x_units || curr_y_unit < 0 || curr_y_unit >= self.y_units {
                if !visited_ids.contains(&[usize::MAX, usize::MAX]) {
                    visited_ids.push([usize::MAX, usize::MAX])
                }
                break 'get_sections;
            }

            [curr_x, curr_y] = self.get_section(curr_x_unit, curr_y_unit).id;
            steps += 1;
            if steps > max_steps {
                break 'get_sections;
            }
        }
        if !visited_ids.contains(&[end_x, end_y]) {
            visited_ids.push([end_x, end_y])
        }
        visited_ids
    }

    pub fn add_ball(&mut self, ball: Ball) {
        let idx = self.ball_id;
        self.ball_id += 1;
        self.ball_cnt += 1;
        self.balls.insert(idx, ball);
        let [ball_x, ball_y] = self.balls.get(&idx).unwrap().position;
        let unit = self.get_section_at_position(ball_x, ball_y);
        unit.items.push(PhysItem::Ball(idx));
        let new_id = unit.id;
        self.balls.get_mut(&idx).unwrap().unit_id = new_id;
        self.balls.get_mut(&idx).unwrap().id = idx;
    }

    pub fn move_ball(&mut self, idx: usize) {
        let ball = self.balls.get(&idx).unwrap();
        let ball_id = ball.unit_id;
        let [ball_x, ball_y] = ball.position;
        let unit = self.get_section_at_position(ball_x, ball_y);
        if unit.id != ball_id {
            unit.items.push(PhysItem::Ball(idx));
            let new_id = unit.id;
            if ball_id[0] >= self.x_units as usize || ball_id[1] >= self.y_units as usize {
                self.out_of_bounds.remove_ball(idx);
            } else {
                self.grid[ball_id[0]][ball_id[1]].remove_ball(idx);
            }
            self.balls.get_mut(&idx).unwrap().unit_id = new_id;
        }
    }

    pub fn add_wall(&mut self, wall: Wall) {
        let idx = self.wall_id;
        self.wall_id += 1;
        self.wall_cnt += 1;
        self.walls.insert(idx, wall);
        let wall_a = self.walls.get(&idx).unwrap().a;
        let wall_b = self.walls.get(&idx).unwrap().b;
        for [unit_x, unit_y] in self.get_sections_between_points(wall_a, wall_b) {
            if unit_x < self.x_units as usize && unit_y < self.y_units as usize {
                self.grid[unit_x][unit_y].items.push(PhysItem::Wall(idx));
            }
        }
        self.walls.get_mut(&idx).unwrap().id = idx;
    }

    pub fn cleanup(&mut self) {
        for item in self.out_of_bounds.items.clone() {
            match item {
                PhysItem::Ball(idx) => {
                    self.out_of_bounds.remove_ball(idx);
                    self.balls.remove(&idx);
                    self.ball_cnt -= 1;
                },
                _ => {continue;},
            }
        }
        let y = (self.y_units - 1) as usize;
        for x in 0..self.x_units as usize {
            for item in self.grid[x][y].items.clone() {
                match item {
                    PhysItem::Ball(idx) => {
                        self.grid[x][y].remove_ball(idx);
                        self.balls.remove(&idx);
                        self.ball_cnt -= 1;
                    },
                    _ => {continue;},
                }
            }
        }
    }

    pub fn handle_collisions(&mut self) {
        for idx in 0..self.ball_id {
            let ball = match self.balls.get(&idx) {
                Some(b) => b,
                None => continue,
            };
            let x_unit = (ball.position[0] as i32 + self.unit_width) / self.unit_width;
            let y_unit = (ball.position[1] as i32 + self.unit_height) / self.unit_height;
            let mut handled = vec![idx];
            for x in (x_unit - 1)..(x_unit + 2) {
                for y in (y_unit - 1)..(y_unit + 2) {
                    if x >= 0 && x < self.x_units && y >= 0 && y < self.y_units {
                        for item in self.grid[x as usize][y as usize].items.clone() {
                            match item {
                                PhysItem::Ball(o_idx) => {
                                    if !handled.contains(&o_idx) {
                                        let [Some(ball), Some(other)] = self.balls.get_disjoint_mut([&idx, &o_idx]) else {
                                            continue;
                                        };
                                        ball.ball_collision(other);
                                        handled.push(o_idx);
                                    }
                                },
                                PhysItem::Wall(o_idx) => {
                                    if !handled.contains(&o_idx) {
                                        let other = self.walls.get(&o_idx).unwrap();
                                        let ball = self.balls.get_mut(&idx).unwrap();
                                        ball.wall_collision(other);
                                        handled.push(o_idx);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn draw_frame<T: RenderTarget>(&mut self, canvas:&mut Canvas<T>, dt: f32) {
        for idx in 0..self.wall_id {
            let wall = match self.walls.get_mut(&idx) {
                Some(w) => w,
                None => continue,
            };
            wall.draw(canvas);
        }

        for idx in 0..self.ball_id {
            let ball = match self.balls.get_mut(&idx) {
                Some(b) => b,
                None => continue,
            };
            ball.move_ball(Some(dt));
            ball.draw(canvas);
            ball.apply_force(GRAVITY, Some(dt));
            self.move_ball(idx);
        }
    }

    pub fn update_boxes(&self, canvas: &mut Canvas<Window>, boxes: &mut Vec<i32>, font: &Font, box_size: u32, window_height: u32) {
        for idx in 0..self.ball_id {
            let position = match self.balls.get(&idx) {
                Some(b) => b.position,
                None => continue,
            };
            if position[1] > window_height as f32 {
                let box_pos = position[0] as i32 / box_size as i32;
                if box_pos >= 0 && box_pos < boxes.len() as i32 {
                    boxes[box_pos as usize] += 1;
                }
            }
        }
        for i in 0..boxes.len() {
            let num_str = boxes[i].to_string();
            let text_surface = font.render(&num_str).blended(Color::RGB(255, 255, 255)).unwrap();
            let texture_creator = canvas.texture_creator();
            let text_texture = texture_creator.create_texture_from_surface(&text_surface).unwrap();
            let texture_query = text_texture.query();
            let x = box_size as i32 * i as i32 + box_size as i32 / 3;
            let y = window_height as i32 - 60;
            let target_rect = Rect::new(x, y, texture_query.width, texture_query.height);
            canvas.copy_ex(&text_texture, None, Some(target_rect), -90.0, None, false, false).unwrap();
        }
    }
}
