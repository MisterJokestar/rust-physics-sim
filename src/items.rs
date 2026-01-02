use rphys::library::*;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::gfx::primitives::DrawRenderer;

/// Maximum allowed velocity for balls (prevents extreme speeds)
const MAX_VELOCITY: f32 = 2000.0;
/// Minimum allowed velocity for balls (prevents extreme speeds)
const MIN_VELOCITY: f32 = -2000.0;
/// Global gravity force vector applied to all balls [x, y]
pub const GRAVITY: [f32; 2] = [0.0, 400.0];

/// Converts SDL2 Color from RGBA to ABGR format for rendering.
///
/// SDL2's gfx primitives expect colors in ABGR format rather than RGBA.
fn to_abgr(color: Color) -> Color {
    Color::RGBA(color.a, color.b, color.g, color.r)
}

/// Represents a physics item in the simulation.
///
/// This enum is used to identify and differentiate between different types of
/// physics objects stored in grid sections. The usize value is the unique ID
/// of the item in the Grid's HashMap.
#[derive(Clone)]
pub enum PhysItem {
    /// A wall object identified by its unique ID
    Wall(usize),
    /// A ball object identified by its unique ID
    Ball(usize),
}

/// Represents a wall (line segment) in the physics simulation.
///
/// Walls are static line segments that balls can collide with. They have
/// physical properties like friction and restitution that affect collision behavior.
pub struct Wall {
    /// Unique identifier for this wall
    pub id: usize,
    /// Starting point of the wall segment [x, y]
    pub a: [f32; 2],
    /// Ending point of the wall segment [x, y]
    pub b: [f32; 2],
    /// Visual width of the wall in pixels
    pub width: i32,
    /// Color used to render the wall
    pub color: Color,
    /// Normalized direction vector along the wall
    pub vec: [f32; 2],
    /// Length of the wall segment
    pub length: f32,
    /// Normalized normal vector (perpendicular to the wall)
    pub nvec: [f32; 2],
    /// Friction coefficient (affects tangential velocity loss in collisions)
    pub friction: f32,
    /// Restitution coefficient (affects normal velocity bounce in collisions)
    pub restitution: f32,
}

impl Wall {
    /// Creates a new wall from two endpoints.
    ///
    /// # Arguments
    ///
    /// * `a` - Starting point of the wall [x, y]
    /// * `b` - Ending point of the wall [x, y]
    /// * `width` - Optional width in pixels (default: 10)
    /// * `color` - Optional color (default: GREEN)
    /// * `friction` - Optional friction coefficient (default: 0.1)
    /// * `restitution` - Optional restitution coefficient (default: 0.1)
    ///
    /// # Returns
    ///
    /// A new Wall instance with computed direction and normal vectors
    pub fn new(
        a: [f32; 2],
        b: [f32; 2],
        width: Option<i32>,
        color: Option<Color>,
        friction: Option<f32>,
        restitution: Option<f32>,
    ) -> Wall {
        let vector = find_vector(a, b);
        let wall = Wall {
            id: 0,
            a: a,
            b: b,
            width: width.unwrap_or(10),
            color: color.unwrap_or(Color::GREEN),
            vec: normalize(vector),
            length: get_magnitude(vector),
            nvec: find_normal(a, b),
            friction: friction.unwrap_or(0.1),
            restitution: restitution.unwrap_or(0.1),
        };
        wall
    }

    /// Draws the wall on the canvas as a thick line.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL2 canvas to draw on
    pub fn draw<T: RenderTarget>(&self, canvas:&mut Canvas<T>) {
        let x1 = self.a[0] as i16;
        let y1 = self.a[1] as i16;
        let x2 = self.b[0] as i16;
        let y2 = self.b[1] as i16;
        let width = self.width as u8;
        let color = to_abgr(self.color);
        let _ = canvas.thick_line(x1, y1, x2, y2, width, color);
    }
}

/// Represents a ball (circle) in the physics simulation.
///
/// Balls are dynamic physics objects that move, collide with walls and other balls,
/// and respond to forces like gravity.
pub struct Ball {
    /// Unique identifier for this ball
    pub id: usize,
    /// Current position in world space [x, y]
    pub position: [f32; 2],
    /// Current velocity vector [vx, vy]
    pub velocity: [f32; 2],
    /// Radius of the ball in pixels
    pub radius: i32,
    /// Color used to render the ball
    pub color: Color,
    /// Friction coefficient (affects tangential velocity loss in collisions)
    pub friction: f32,
    /// Restitution coefficient (affects normal velocity bounce in collisions)
    pub restitution: f32,
    /// ID of the grid section this ball currently occupies [x_unit, y_unit]
    pub unit_id: [usize; 2],
}

impl Ball {
    /// Creates a new ball at the specified position.
    ///
    /// # Arguments
    ///
    /// * `position` - Initial position [x, y]
    /// * `velocity` - Optional initial velocity [vx, vy] (default: [0.0, 0.0])
    /// * `radius` - Optional radius in pixels (default: 10)
    /// * `color` - Optional color (default: RED)
    /// * `friction` - Optional friction coefficient (default: 0.1)
    /// * `restitution` - Optional restitution coefficient (default: 0.1)
    ///
    /// # Returns
    ///
    /// A new Ball instance
    pub fn new(
        position: [f32; 2],
        velocity: Option<[f32; 2]>,
        radius: Option<i32>,
        color: Option<Color>,
        friction: Option<f32>,
        restitution: Option<f32>,
    ) -> Ball {
        let ball = Ball {
            id: 0,
            position: position,
            velocity: velocity.unwrap_or([0.0, 0.0]),
            radius: radius.unwrap_or(10),
            color: color.unwrap_or(Color::RED),
            friction: friction.unwrap_or(0.1),
            restitution: restitution.unwrap_or(0.1),
            unit_id: [0, 0]
        };
        ball
    }

    /// Draws the ball on the canvas as a filled circle.
    ///
    /// # Arguments
    ///
    /// * `canvas` - The SDL2 canvas to draw on
    pub fn draw<T: RenderTarget>(&self, canvas:&mut Canvas<T>) {
        let x = self.position[0] as i16;
        let y = self.position[1] as i16;
        let rad = self.radius as i16;
        let color = to_abgr(self.color);
        let _ = canvas.filled_circle(x, y, rad, color);
    }

    /// Updates the ball's position based on its velocity.
    ///
    /// Applies velocity to position using: `position += velocity * dt`
    /// Also clamps velocity to prevent extreme speeds.
    ///
    /// # Arguments
    ///
    /// * `delta` - Optional time delta in seconds (default: 1.0)
    pub fn move_ball(&mut self, delta: Option<f32>) {
        let dt = delta.unwrap_or(1.0);
        let new_x = self.position[0] + self.velocity[0] * dt;
        let new_y = self.position[1] + self.velocity[1] * dt;
        // Clamp velocities to prevent unrealistic speeds
        let clamped_vx = self.velocity[0].clamp(MIN_VELOCITY, MAX_VELOCITY);
        let clamped_vy = self.velocity[1].clamp(MIN_VELOCITY, MAX_VELOCITY);
        self.position = [new_x, new_y];
        self.velocity = [clamped_vx, clamped_vy];
    }

    /// Applies a force to the ball, modifying its velocity.
    ///
    /// Uses simple Euler integration: `velocity += force * dt`
    ///
    /// # Arguments
    ///
    /// * `force` - The force vector to apply [fx, fy]
    /// * `delta` - Optional time delta in seconds (default: 1.0)
    pub fn apply_force(&mut self, force: [f32; 2], delta: Option<f32>) {
        let dt = delta.unwrap_or(1.0);
        let new_x = self.velocity[0] + force[0] * dt;
        let new_y = self.velocity[1] + force[1] * dt;
        self.velocity = [new_x, new_y];
    }

    /// Handles collision between this ball and a wall.
    ///
    /// Uses impulse-based collision resolution with friction and restitution.
    /// Handles both line segment collisions and endpoint (corner) collisions.
    ///
    /// # Arguments
    ///
    /// * `wall` - The wall to check collision with
    pub fn wall_collision(&mut self, wall:&Wall) {
        // Find vector from wall start to ball
        let mut vec = find_vector(wall.a, self.position);
        // Project ball position onto wall direction to find closest point
        let position = dot(vec, wall.vec);
        let nv: [f32; 2];  // Normal vector (perpendicular to collision surface)
        let tv: [f32; 2];  // Tangent vector (along collision surface)
        let dist: f32;     // Distance from ball to wall/endpoint
        let min_dist: f32; // Minimum distance before collision

        // If ball is past the end of the wall, check collision with endpoint
        if position > wall.length {
            vec = find_vector(wall.b, self.position)
        }

        // Check if ball is colliding with wall endpoints (corners)
        if position < 0.0 || position > wall.length {
            // Endpoint collision: use radial normal from endpoint to ball center
            nv = normalize(vec);
            tv = [-nv[1], nv[0]];
            dist = get_magnitude(vec);
            min_dist = self.radius as f32;
        } else {
            // Line segment collision: use wall's normal vector
            nv = wall.nvec;
            tv = wall.vec;
            dist = dot(vec, wall.nvec);
            min_dist = (self.radius + wall.width / 2) as f32;
        }

        // Early exit if ball is too far from wall
        if dist.abs() > min_dist {
            return;
        }

        // Calculate velocity components along normal and tangent
        let n_vel = dot(self.velocity, nv);
        // Early exit if ball is moving away from wall
        if (n_vel < 0.0 && dist < 0.0) || (n_vel > 0.0 && dist > 0.0) {
            return;
        }
        let t_vel = dot(self.velocity, tv);

        // Apply physics: bounce (restitution) and friction
        let total_restitution = wall.restitution + self.restitution;
        let total_friction = wall.friction + self.friction;

        // Calculate new velocity components
        // Normal component: reversed and scaled by restitution (bounce)
        let x_n = -n_vel * wall.nvec[0] * total_restitution;
        // Tangent component: preserved but reduced by friction
        let x_t = t_vel * wall.vec[0] * (1.0 - total_friction);
        let y_n = -n_vel * wall.nvec[1] * total_restitution;
        let y_t = t_vel * wall.vec[1] * (1.0 - total_friction);
        self.velocity = [x_n + x_t, y_n + y_t];

        // Resolve penetration by pushing ball out of wall
        let penetration = min_dist - dist.abs();
        if penetration > 0.0 {
            let sign = if dist >= 0.0 {1.0} else {-1.0};
            let new_x = self.position[0] + nv[0] * penetration * sign;
            let new_y = self.position[1] + nv[1] * penetration * sign;
            self.position = [new_x, new_y];
        }
    }

    /// Handles collision between this ball and another ball.
    ///
    /// Uses impulse-based collision resolution with friction and restitution.
    /// Updates velocities and positions of both balls.
    ///
    /// # Arguments
    ///
    /// * `other` - The other ball to collide with (mutable reference)
    pub fn ball_collision(&mut self, other:&mut Ball) {
        // Find vector from other ball to this ball
        let vec = find_vector(other.position, self.position);
        let dist = get_magnitude(vec);
        let min_dist = (self.radius + other.radius) as f32;

        // Early exit if balls aren't touching
        if dist > min_dist {
            return;
        }

        // Calculate collision normal and tangent vectors
        let nv = normalize(vec);  // Normal: from other to self
        let tv = [-nv[1], nv[0]]; // Tangent: perpendicular to normal

        // Decompose velocities into normal and tangential components
        let n_vel_self = dot(self.velocity, nv);
        let t_vel_self = dot(self.velocity, tv);
        let n_vel_other = dot(other.velocity, nv);
        let t_vel_other = dot(other.velocity, tv);

        // Early exit if balls are moving apart (not approaching each other)
        if n_vel_self - n_vel_other > 0.0 {
            return;
        }

        // Calculate average normal velocity for equal mass collision
        let avg_n_vel = (n_vel_self.abs() + n_vel_other.abs()) / 2.0;
        let total_restitution = self.restitution + other.restitution;
        let total_friction = self.friction + other.friction;

        // Calculate new velocity components
        // Normal component: reversed for both balls (equal and opposite)
        let x_n = avg_n_vel * nv[0] * total_restitution;
        // Tangent component: preserved but reduced by friction
        let x_t_self = t_vel_self * tv[0] * (1.0 - total_friction);
        let x_t_other = t_vel_other * tv[0] * (1.0 - total_friction);
        let y_n = avg_n_vel * nv[1] * total_restitution;
        let y_t_self = t_vel_self * tv[1] * (1.0 - total_friction);
        let y_t_other = t_vel_other * tv[1] * (1.0 - total_friction);

        // Apply new velocities (normal components are opposite for each ball)
        self.velocity = [x_n + x_t_self, y_n + y_t_self];
        other.velocity = [-x_n + x_t_other, -y_n + y_t_other];

        // Resolve penetration by pushing balls apart equally
        let penetration = min_dist - dist.abs();
        if penetration > 0.0 {
            // Each ball moves half the penetration distance
            let new_x_self = self.position[0] + nv[0] * penetration / 2.0;
            let new_y_self = self.position[1] + nv[1] * penetration / 2.0;
            let new_x_other = other.position[0] - nv[0] * penetration / 2.0;
            let new_y_other = other.position[1] - nv[1] * penetration / 2.0;
            self.position = [new_x_self, new_y_self];
            other.position = [new_x_other, new_y_other];
        }
    }
}
