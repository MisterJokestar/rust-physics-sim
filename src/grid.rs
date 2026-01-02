use rphys::library::*;
use crate::items::{PhysItem, Ball, Wall, GRAVITY};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::ttf::Font;
use std::collections::HashMap;

/// A spatial partitioning grid for efficient collision detection.
///
/// The grid divides the simulation space into uniform cells (sections) and tracks
/// which physics objects are in which cells. This allows collision detection to only
/// check nearby objects rather than all pairs.
pub struct Grid {
    /// Width of each grid cell in pixels
    unit_width: i32,
    /// Height of each grid cell in pixels
    unit_height: i32,
    /// 2D array of grid sections
    grid: Vec<Vec<Section>>,
    /// Special section for objects outside the grid bounds
    out_of_bounds: Section,
    /// All walls in the simulation, indexed by unique ID
    walls: HashMap<usize, Wall>,
    /// All balls in the simulation, indexed by unique ID
    balls: HashMap<usize, Ball>,
    /// Number of grid cells horizontally
    x_units: i32,
    /// Number of grid cells vertically
    y_units: i32,
    /// Current count of active balls
    ball_cnt: usize,
    /// Next available ball ID (monotonically increasing)
    ball_id: usize,
    /// Current count of active walls
    wall_cnt: usize,
    /// Next available wall ID (monotonically increasing)
    wall_id: usize,
}

/// A single cell in the spatial partitioning grid.
///
/// Each section tracks which physics items (balls and walls) are currently
/// within its spatial bounds.
pub struct Section {
    /// Grid coordinates of this section [x, y]
    pub id: [usize; 2],
    /// Physics items currently in this section
    pub items: Vec<PhysItem>,
}

impl Section {
    /// Removes a ball from this section by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique ID of the ball to remove
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
    /// Creates a new spatial partitioning grid.
    ///
    /// # Arguments
    ///
    /// * `unit_width` - Width of each grid cell in pixels
    /// * `unit_height` - Height of each grid cell in pixels
    /// * `window_width` - Total width of the simulation window
    /// * `window_height` - Total height of the simulation window
    ///
    /// # Returns
    ///
    /// A new Grid instance with all sections initialized
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
        // Initialize all grid sections
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

    /// Gets a mutable reference to a section by grid coordinates.
    ///
    /// Returns the out_of_bounds section if coordinates are invalid.
    ///
    /// # Arguments
    ///
    /// * `x` - Grid x-coordinate
    /// * `y` - Grid y-coordinate
    pub fn get_section(&mut self, x: i32, y: i32) -> &mut Section {
        if x < 0 || x >= self.x_units || y < 0 || y >= self.y_units {
            return &mut self.out_of_bounds;
        }
        &mut self.grid[x as usize][y as usize]
    }

    /// Gets a mutable reference to a section by world position.
    ///
    /// Converts world coordinates to grid coordinates and returns the section.
    ///
    /// # Arguments
    ///
    /// * `x` - World x-coordinate in pixels
    /// * `y` - World y-coordinate in pixels
    pub fn get_section_at_position(&mut self, x: f32, y: f32) -> &mut Section {
        let x_unit = (x as i32 + self.unit_width) / self.unit_width;
        let y_unit = (y as i32 + self.unit_height) / self.unit_height;
        self.get_section(x_unit, y_unit)
    }

    /// Finds all grid sections that a line segment passes through.
    ///
    /// Uses a DDA-like (Digital Differential Analyzer) algorithm to trace a line
    /// through the grid and identify all sections it intersects. This is used when
    /// adding walls to register them in all relevant sections.
    ///
    /// # Arguments
    ///
    /// * `s` - Starting point of the line segment [x, y]
    /// * `e` - Ending point of the line segment [x, y]
    ///
    /// # Returns
    ///
    /// A vector of section IDs that the line passes through
    pub fn get_sections_between_points(&mut self, s: [f32; 2], e: [f32; 2]) -> Vec<[usize; 2]> {
        // Normalize direction vector from start to end
        let vec = normalize(find_vector(s, e));
        let vx = vec[0];
        let vy = vec[1];

        // Convert starting position to grid coordinates
        let mut curr_x_unit = (s[0] as i32 + self.unit_width) / self.unit_width;
        let mut curr_y_unit = (s[1] as i32 + self.unit_height) / self.unit_height;
        // Position relative to current grid cell
        let mut relative_x = s[0] % self.unit_width as f32;
        let mut relative_y = s[1] % self.unit_height as f32;

        let [mut curr_x, mut curr_y] = self.get_section(curr_x_unit, curr_y_unit).id;
        let [end_x, end_y] = self.get_section_at_position(e[0], e[1]).id;

        // Early return if no direction or already at destination
        if (vx == 0.0 && vy == 0.0) || (curr_x == end_x && curr_y == end_y) {
            return vec![[curr_x, curr_y]];
        }

        let mut visited_ids: Vec<[usize; 2]> = Vec::new();

        // Safety limit to prevent infinite loops
        let max_steps = (self.x_units + self.y_units) * 2 + 10;
        let mut steps = 0;

        // DDA-like line traversal algorithm
        'get_sections: loop {
            if !visited_ids.contains(&[curr_x, curr_y]) {
                visited_ids.push([curr_x, curr_y]);
            }

            if curr_x == end_x && curr_y == end_y {
                break 'get_sections;
            }

            // Calculate time to reach next horizontal grid boundary
            let mut tx: f32;
            if vx < 0.0 {
                tx = (-relative_x) / vx;
            } else if vx > 0.0 {
                tx = (self.unit_width as f32 - relative_x) / vx;
            } else {
                tx = f32::INFINITY;
            }

            // Calculate time to reach next vertical grid boundary
            let mut ty: f32;
            if vy < 0.0 {
                ty = (-relative_y) / vy;
            } else if vy > 0.0 {
                ty = (self.unit_height as f32 - relative_y) / vy;
            } else {
                ty = f32::INFINITY;
            }

            // Ignore negative or zero times (rounding errors)
            if tx <= 0.0 { tx = f32::INFINITY; }
            if ty <= 0.0 { ty = f32::INFINITY; }

            // Move to whichever boundary is closer
            let t: f32;
            if tx < ty {
                // Cross vertical boundary first
                t = tx;
                if vx > 0.0 { curr_x_unit += 1 } else { curr_x_unit -= 1 };

                relative_x = if vx > 0.0 { 0.0 } else { self.unit_width as f32 };
                relative_y = vy * t;
            } else {
                // Cross horizontal boundary first
                t = ty;
                if vy > 0.0 { curr_y_unit += 1 } else { curr_y_unit -= 1 };

                relative_x = vx * t;
                relative_y = if vy > 0.0 { 0.0 } else { self.unit_height as f32 };
            }

            // Check if we've gone out of bounds
            if curr_x_unit < 0 || curr_x_unit >= self.x_units || curr_y_unit < 0 || curr_y_unit >= self.y_units {
                if !visited_ids.contains(&[usize::MAX, usize::MAX]) {
                    visited_ids.push([usize::MAX, usize::MAX])
                }
                break 'get_sections;
            }

            [curr_x, curr_y] = self.get_section(curr_x_unit, curr_y_unit).id;
            steps += 1;
            // Safety check to prevent infinite loops
            if steps > max_steps {
                break 'get_sections;
            }
        }
        // Ensure end section is included
        if !visited_ids.contains(&[end_x, end_y]) {
            visited_ids.push([end_x, end_y])
        }
        visited_ids
    }

    /// Adds a new ball to the grid.
    ///
    /// Assigns a unique ID to the ball, adds it to the balls HashMap, and registers
    /// it in the appropriate grid section based on its position.
    ///
    /// # Arguments
    ///
    /// * `ball` - The ball to add
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

    /// Updates a ball's grid section if it has moved to a new section.
    ///
    /// Removes the ball from its old section and adds it to its new section
    /// based on its current position.
    ///
    /// # Arguments
    ///
    /// * `idx` - The unique ID of the ball to update
    pub fn move_ball(&mut self, idx: usize) {
        let ball = self.balls.get(&idx).unwrap();
        let ball_id = ball.unit_id;
        let [ball_x, ball_y] = ball.position;
        let unit = self.get_section_at_position(ball_x, ball_y);
        // Only update if ball has moved to a different section
        if unit.id != ball_id {
            unit.items.push(PhysItem::Ball(idx));
            let new_id = unit.id;
            // Remove from old section
            if ball_id[0] >= self.x_units as usize || ball_id[1] >= self.y_units as usize {
                self.out_of_bounds.remove_ball(idx);
            } else {
                self.grid[ball_id[0]][ball_id[1]].remove_ball(idx);
            }
            self.balls.get_mut(&idx).unwrap().unit_id = new_id;
        }
    }

    /// Adds a new wall to the grid.
    ///
    /// Assigns a unique ID to the wall, adds it to the walls HashMap, and registers
    /// it in all grid sections that the wall line segment passes through.
    ///
    /// # Arguments
    ///
    /// * `wall` - The wall to add
    pub fn add_wall(&mut self, wall: Wall) {
        let idx = self.wall_id;
        self.wall_id += 1;
        self.wall_cnt += 1;
        self.walls.insert(idx, wall);
        let wall_a = self.walls.get(&idx).unwrap().a;
        let wall_b = self.walls.get(&idx).unwrap().b;
        // Register wall in all sections it passes through
        for [unit_x, unit_y] in self.get_sections_between_points(wall_a, wall_b) {
            if unit_x < self.x_units as usize && unit_y < self.y_units as usize {
                self.grid[unit_x][unit_y].items.push(PhysItem::Wall(idx));
            }
        }
        self.walls.get_mut(&idx).unwrap().id = idx;
    }

    /// Removes balls that are out of bounds or at the bottom of the simulation.
    ///
    /// This cleanup is typically called once per frame to remove balls that have
    /// left the play area or reached the collection zones.
    pub fn cleanup(&mut self) {
        // Remove balls that went out of bounds
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
        // Remove balls at the bottom row (collection zone)
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

    /// Handles all collisions between balls and between balls and walls.
    ///
    /// Uses the spatial partitioning grid to efficiently check only nearby objects.
    /// For each ball, checks the 3x3 grid of sections around it for potential collisions.
    pub fn handle_collisions(&mut self) {
        for idx in 0..self.ball_id {
            let ball = match self.balls.get(&idx) {
                Some(b) => b,
                None => continue,
            };
            // Get ball's current grid position
            let x_unit = (ball.position[0] as i32 + self.unit_width) / self.unit_width;
            let y_unit = (ball.position[1] as i32 + self.unit_height) / self.unit_height;
            // Track which items we've already checked to avoid duplicate collisions
            let mut handled = vec![idx];

            // Check 3x3 grid of sections around the ball
            for x in (x_unit - 1)..(x_unit + 2) {
                for y in (y_unit - 1)..(y_unit + 2) {
                    if x >= 0 && x < self.x_units && y >= 0 && y < self.y_units {
                        for item in self.grid[x as usize][y as usize].items.clone() {
                            match item {
                                PhysItem::Ball(o_idx) => {
                                    if !handled.contains(&o_idx) {
                                        // Get mutable references to both balls
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

    /// Renders all physics objects and updates ball physics for this frame.
    ///
    /// Updates ball positions, applies gravity, handles rendering, and updates
    /// grid sections as balls move.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL2 canvas to draw on
    /// * `dt` - Time delta in seconds since last frame
    pub fn draw_frame<T: RenderTarget>(&mut self, canvas:&mut Canvas<T>, dt: f32) {
        // Draw all walls
        for idx in 0..self.wall_id {
            let wall = match self.walls.get_mut(&idx) {
                Some(w) => w,
                None => continue,
            };
            wall.draw(canvas);
        }

        // Update and draw all balls
        for idx in 0..self.ball_id {
            let ball = match self.balls.get_mut(&idx) {
                Some(b) => b,
                None => continue,
            };
            ball.move_ball(Some(dt));
            ball.draw(canvas);
            ball.apply_force(GRAVITY, Some(dt));
            // Update which grid section the ball is in
            self.move_ball(idx);
        }
    }

    /// Updates and renders the Plinko collection box counts.
    ///
    /// Counts balls that have reached the bottom and updates the display showing
    /// how many balls have landed in each collection box.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL2 canvas to draw on
    /// * `boxes` - Vector tracking ball count for each collection box
    /// * `font` - Font to use for rendering numbers
    /// * `box_size` - Width of each collection box in pixels
    /// * `window_height` - Height of the window in pixels
    pub fn update_boxes(&self, canvas: &mut Canvas<Window>, boxes: &mut Vec<i32>, font: &Font, box_size: u32, window_height: u32) {
        // Count balls that have reached the bottom
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
        // Render the count for each box
        for i in 0..boxes.len() {
            let num_str = boxes[i].to_string();
            let text_surface = font.render(&num_str).blended(Color::RGB(255, 255, 255)).unwrap();
            let texture_creator = canvas.texture_creator();
            let text_texture = texture_creator.create_texture_from_surface(&text_surface).unwrap();
            let texture_query = text_texture.query();
            let x = box_size as i32 * i as i32 + box_size as i32 / 3;
            let y = window_height as i32 - 60;
            let target_rect = Rect::new(x, y, texture_query.width, texture_query.height);
            // Rotate text 90 degrees for vertical display
            canvas.copy_ex(&text_texture, None, Some(target_rect), -90.0, None, false, false).unwrap();
        }
    }
}
