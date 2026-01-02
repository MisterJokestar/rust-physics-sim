use rphys::library::*;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::gfx::primitives::DrawRenderer;

const MAX_VELOCITY: f32 = 2000.0;
const MIN_VELOCITY: f32 = -2000.0;
pub const GRAVITY: [f32; 2] = [0.0, 400.0];

fn to_abgr(color: Color) -> Color {
    Color::RGBA(color.a, color.b, color.g, color.r)
}

#[derive(Clone)]
pub enum PhysItem {
    Wall(usize),
    Ball(usize),
}

pub struct Wall {
    pub id: usize,
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub width: i32,
    pub color: Color,
    pub vec: [f32; 2],
    pub length: f32,
    pub nvec: [f32; 2],
    pub friction: f32,
    pub restitution: f32,
}

impl Wall {
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

pub struct Ball {
    pub id: usize,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub radius: i32,
    pub color: Color,
    pub friction: f32,
    pub restitution: f32,
    pub unit_id: [usize; 2],
}

impl Ball {
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

    pub fn draw<T: RenderTarget>(&self, canvas:&mut Canvas<T>) {
        let x = self.position[0] as i16;
        let y = self.position[1] as i16;
        let rad = self.radius as i16;
        let color = to_abgr(self.color);
        let _ = canvas.filled_circle(x, y, rad, color);
    }

    pub fn move_ball(&mut self, delta: Option<f32>) {
        let dt = delta.unwrap_or(1.0);
        let new_x = self.position[0] + self.velocity[0] * dt;
        let new_y = self.position[1] + self.velocity[1] * dt;
        let clamped_vx = self.velocity[0].clamp(MIN_VELOCITY, MAX_VELOCITY);
        let clamped_vy = self.velocity[1].clamp(MIN_VELOCITY, MAX_VELOCITY);
        self.position = [new_x, new_y];
        self.velocity = [clamped_vx, clamped_vy];
    }

    pub fn apply_force(&mut self, force: [f32; 2], delta: Option<f32>) {
        let dt = delta.unwrap_or(1.0);
        let new_x = self.velocity[0] + force[0] * dt;
        let new_y = self.velocity[1] + force[1] * dt;
        self.velocity = [new_x, new_y];
    }

    pub fn wall_collision(&mut self, wall:&Wall) {
        let mut vec = find_vector(wall.a, self.position);
        let position = dot(vec, wall.vec);
        let nv: [f32; 2];
        let tv: [f32; 2];
        let dist: f32;
        let min_dist: f32;
        if position > wall.length {
            vec = find_vector(wall.b, self.position)
        }
        if position < 0.0 || position > wall.length {
            nv = normalize(vec);
            tv = [-nv[1], nv[0]];
            dist = get_magnitude(vec);
            min_dist = self.radius as f32;
        } else {
            nv = wall.nvec;
            tv = wall.vec;
            dist = dot(vec, wall.nvec);
            min_dist = (self.radius + wall.width / 2) as f32;
        }
        if dist.abs() > min_dist {
            return;
        }
        let n_vel = dot(self.velocity, nv);
        if (n_vel < 0.0 && dist < 0.0) || (n_vel > 0.0 && dist > 0.0) {
            return;
        }
        let t_vel = dot(self.velocity, tv);
        let total_restitution = wall.restitution + self.restitution;
        let total_friction = wall.friction + self.friction;
        let x_n = -n_vel * wall.nvec[0] * (1.0 - total_restitution);
        let x_t = t_vel * wall.vec[0] * (1.0 - total_friction);
        let y_n = -n_vel * wall.nvec[1] * (1.0 - total_restitution);
        let y_t = t_vel * wall.vec[1] * (1.0 - total_friction);
        self.velocity = [x_n + x_t, y_n + y_t];
        let penetration = min_dist - dist.abs();
        if penetration > 0.0 {
            let sign = if dist >= 0.0 {1.0} else {-1.0};
            let new_x = self.position[0] + nv[0] * penetration * sign;
            let new_y = self.position[1] + nv[1] * penetration * sign;
            self.position = [new_x, new_y];
        }
    }

    pub fn ball_collision(&mut self, other:&mut Ball) {
        let vec = find_vector(other.position, self.position);
        let dist = get_magnitude(vec);
        let min_dist = (self.radius + other.radius) as f32;
        if dist > min_dist {
            return;
        }
        let nv = normalize(vec);
        let tv = [-nv[1], nv[0]];
        let n_vel_self = dot(self.velocity, nv);
        let t_vel_self = dot(self.velocity, tv);
        let n_vel_other = dot(other.velocity, nv);
        let t_vel_other = dot(other.velocity, tv);
        if n_vel_self - n_vel_other > 0.0 {
            return;
        }
        let avg_n_vel = (n_vel_self.abs() + n_vel_other.abs()) / 2.0;
        let total_restitution = self.restitution + other.restitution;
        let total_friction = self.friction + other.friction;
        let x_n = avg_n_vel * nv[0] * (1.0 - total_restitution);
        let x_t_self = t_vel_self * tv[0] * (1.0 - total_friction);
        let x_t_other = t_vel_other * tv[0] * (1.0 - total_friction);
        let y_n = avg_n_vel * nv[1] * (1.0 - total_restitution);
        let y_t_self = t_vel_self * tv[1] * (1.0 - total_friction);
        let y_t_other = t_vel_other * tv[1] * (1.0 - total_friction);
        self.velocity = [x_n + x_t_self, y_n + y_t_self];
        other.velocity = [-x_n + x_t_other, -y_n + y_t_other];
        let penetration = min_dist - dist.abs();
        if penetration > 0.0 {
            let new_x_self = self.position[0] + nv[0] * penetration / 2.0;
            let new_y_self = self.position[1] + nv[1] * penetration / 2.0;
            let new_x_other = other.position[0] - nv[0] * penetration / 2.0;
            let new_y_other = other.position[1] - nv[1] * penetration / 2.0;
            self.position = [new_x_self, new_y_self];
            other.position = [new_x_other, new_y_other];
        }
    }
}
