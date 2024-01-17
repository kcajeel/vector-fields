// #![allow(unreachable_code, unused_variables, dead_code)]

use std::cell::RefCell;

use nannou::{
    noise::{NoiseFn, Perlin},
    prelude::*,
};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    particles: Particles,
    vector_field: VectorField,
}

fn model(app: &App) -> Model {
    app.new_window().fullscreen().view(view).build().unwrap();

    let mut particles = Particles::new(300, 6.0);
    particles.init();

    let mut vector_field = VectorField::new(20.0);
    vector_field.init_vectors();

    Model {
        particles,
        vector_field,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.particles.run_simulation(&model.vector_field);
    model.vector_field.update();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let particle_color = (1.0, 1.0, 1.0, 0.4);
    let vector_field_color = (0.3, 0.3, 0.3);

    draw.background().rgb(0.2, 0.2, 0.2);

    model.particles.display(&draw, particle_color);
    model.vector_field.display(&draw, vector_field_color);
    // draw.ellipse()
    // .xy(app.mouse.position())
    // .rgba(255.0,0.0,0.0,100.0);
    // put everything on the frame
    draw.to_frame(app, &frame).unwrap();
}

struct Particle {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    last_position: RefCell<Vec2>,
    max_speed: f32,
}

impl Particle {
    pub fn new(speed_limit: f32) -> Particle {
        let position = Vec2::new(random_range(-768.0, 768.0), random_range(-450.0, 450.0));
        Self {
            position: position,
            velocity: Vec2::new(0.0, 0.0),
            acceleration: Vec2::new(0.0, 0.0),
            last_position: RefCell::new(position),
            max_speed: speed_limit,
        }
    }

    pub fn update(&mut self) {
        self.position += self.velocity;
        self.velocity += self.acceleration;
        self.acceleration *= 0.0;

        self.limit();
        self.keep_on_screen();
    }

    fn limit(&mut self) {
        if self.velocity.length() > self.max_speed {
            self.velocity = self.velocity.normalize();
            self.velocity.x *= self.max_speed;
            self.velocity.y *= self.max_speed;
        }
    }

    fn keep_on_screen(&mut self) {
        let width = 768.0;
        let height = 450.0; //TODO: figure out a better way to do this

        if self.position.x > width {
            self.position.x = -width;
            self.update_last_position();
        } else if self.position.x < -width {
            self.position.x = width;
            self.update_last_position();
        }
        if self.position.y > height {
            self.position.y = -height;
            self.update_last_position();
        } else if self.position.y < -height {
            self.position.y = height;
            self.update_last_position();
        }
    }

    pub fn update_last_position(&self) {
        let mut last_position = self.last_position.borrow_mut();
        *last_position = self.position;
    }

    fn apply_force(&mut self, force: Vec2) {
        self.acceleration.x += force.x;
        self.acceleration.y += force.y;
    }

    pub fn follow(&mut self, vector_field: &VectorField) {
        let vector_column = (self.position.x) / vector_field.scale_factor;
        let vector_row = (self.position.y) / vector_field.scale_factor;
        let index = abs(vector_row + vector_column * (vector_field.columns as f32));
        let force = vector_field.vectors[index as usize];
        // println!(
        //     "position: ({},{})\nvector: {}: ({},{})",
        //     self.position.x as i32,
        //     self.position.y as i32,
        //     index as usize,
        //     vector_field.vectors[index as usize].x as i32,
        //     vector_field.vectors[index as usize].y as i32
        // );
        self.apply_force(force);
    }

    pub fn display(&self, draw: &Draw, color: (f32, f32, f32, f32)) {
        let cartesian_position = Vec2::new(self.position.x, -self.position.y);
        let cartesian_last_position = Vec2::new(
            self.last_position.borrow().x,
            -self.last_position.borrow().y,
        );
        let trail = (cartesian_position - cartesian_last_position) * 2.0;

        draw.line()
            .start(cartesian_position)
            .end(Vec2::new(
                cartesian_last_position.x - trail.x,
                cartesian_last_position.y - trail.y,
            ))
            .rgba(color.0, color.1, color.2, color.3)
            .weight(1.0)
            .caps_round();

        // draw.ellipse()
        //     .rgba(255.0, 255.0, 255.0, 200.0)
        //     .x_y(cartesian_position.x, cartesian_position.y)
        //     .radius(5.0);

        self.update_last_position();
    }
}

struct Particles {
    particles: Vec<Particle>,
    max_speed: f32,
}

impl Particles {
    pub fn new(number_of_particles: usize, speed_limit: f32) -> Particles {
        Self {
            particles: Vec::with_capacity(number_of_particles),
            max_speed: speed_limit,
        }
    }

    pub fn init(&mut self) {
        for _ in 0..self.particles.capacity() {
            let mut particle = Particle::new(self.max_speed);
            particle.apply_force(vec2(random_f32(), random_f32()));
            self.particles.push(particle);
        }
    }

    pub fn run_simulation(&mut self, vector_field: &VectorField) {
        for p in &mut self.particles {
            p.follow(vector_field);
            p.update();
        }
    }

    pub fn display(&self, draw: &Draw, color: (f32, f32, f32, f32)) {
        for p in &self.particles {
            p.display(draw, color);
        }
    }
}

struct VectorField {
    columns: i32,
    rows: i32,
    scale_factor: f32,
    noise_seed: Vec3,
    vectors: Vec<Vec2>,
    increment: f32,
}

impl VectorField {
    pub fn new(scale: f32) -> Self {
        Self {
            columns: 77,
            rows: 44,
            scale_factor: scale,
            noise_seed: Vec3::new(0.0, 0.0, 0.0),
            vectors: Vec::new(),
            increment: 0.1,
        }
    }

    pub fn init_vectors(&mut self) {
        let angle_increment = 2.0 * std::f32::consts::PI / (self.columns * self.rows) as f32;

        for i in 0..self.rows {
            for j in 0..self.columns {
                let angle = angle_increment * (i * self.columns + j) as f32;
                let v = Vec2::new(angle.cos(), angle.sin());
                let start = pt2(
                    j as f32 * self.scale_factor - 768.0,
                    -i as f32 * self.scale_factor + 450.0,
                );
                let end = start + v;
                self.vectors.push(end);
            }
        }
    }

    pub fn update(&mut self) {
        self.noise_seed.y = 0.0;
        for i in 0..self.rows {
            self.noise_seed.x = 0.0;
            for j in 0..self.columns {
                let noise = Perlin::new();
                let angle = noise.get([
                    self.noise_seed.x as f64,
                    self.noise_seed.y as f64,
                    self.noise_seed.z as f64,
                ]);

                let mut v = Vec2::new(angle.cos() as f32, angle.sin() as f32);
                v = v.normalize();
                let index = i * self.columns + j;
                self.vectors[index as usize] = v;
                self.noise_seed.x += self.increment;
            }
            self.noise_seed.y += self.increment;
        }
        self.noise_seed.z += 0.009;
    }

    pub fn display(&self, draw: &Draw, color: (f32, f32, f32)) {
        for i in 0..self.rows {
            for j in 0..self.columns {
                let index = i * self.columns + j;
                let v = self.vectors.get(index as usize).unwrap();

                let start = pt2(
                    j as f32 * self.scale_factor - 780.0,
                    -i as f32 * self.scale_factor + 450.0,
                );
                let end = start + vec2(self.scale_factor, 0.0).rotate(v.angle());

                draw.line()
                    .start(start)
                    .end(end)
                    // .rotate(v.angle())
                    .rgb(color.0, color.1, color.2)
                    .weight(1.0);
            }
        }
    }
}
