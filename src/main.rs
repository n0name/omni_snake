extern crate math2d;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use math2d::math::common::*;
use math2d::math::vector::Vec2D;
use math2d::math::angle::Angle;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::collections::VecDeque;
use rand::Rng;

const SEGMENT_RADIUS: f64 = 5.0;            // [Pix]
const APPLE_RADIUS: f64 = 5.0;              // [Pix]
const UPDATE_TIME: f64 = 0.05;              // [S]
const SPAWN_TIME: f64 = 5.0;                // [S]
const SEGMENTS_PER_APPLE: usize = 10;       // [#]
const TURN_ANGLE: f64 = 15.0;               // [Deg]

struct Snake {
    dir: Vec2D,
    segments: VecDeque<Vec2D>,
}

impl Snake {
    fn new(start_pos: Vec2D, start_dir: Vec2D) -> Snake {
        let seg_separation = SEGMENT_RADIUS;
        let mut segments = VecDeque::new();
        segments.push_back(start_pos);
        segments.push_back(start_pos - start_dir * seg_separation);
        segments.push_back(start_pos - start_dir * seg_separation * 2.0);
        Snake {
            dir: start_dir,
            segments: segments
        }
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Snake,
    apples: Vec<Vec2D>,
    last_move: f64,
    last_spawn: f64,
    new_segments: usize,
    score: usize
}

enum CollisionType {
    No,
    Apple,
    Snake
}

impl App {
    fn new(opengl: glutin_window::OpenGL) -> App {
        App {
            gl: GlGraphics::new(opengl),
            snake: Snake::new(Vec2D::new(250.0, 250.0), Vec2D::new(-1.0, 0.0)),
            apples: Vec::new(),
            last_move: 0.0,
            last_spawn: 0.0,
            new_segments: 0,
            score: 0
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let segment_graphics = ellipse::Ellipse::new(GREEN);
        let apple_graphics = ellipse::Ellipse::new(RED);
        let snake = &self.snake;
        let apples = &self.apples;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);

            apples.iter().for_each(|a| {
                let (x, y) = a.as_tuple();
                let transform = c.transform.trans(x - 250.0, y - 250.0);
                apple_graphics.draw(ellipse::circle(x, y, APPLE_RADIUS), &c.draw_state, transform, gl);
            });
            
            snake.segments.iter().for_each(|s|{
                let (x, y) = s.as_tuple();
                let transform = c.transform.trans(x - 250.0, y - 250.0);
                segment_graphics.draw(ellipse::circle(x, y, SEGMENT_RADIUS), &c.draw_state, transform, gl);
            });
        });
    }

    fn check_collisions(&mut self, head: &Vec2D) -> CollisionType {
        let mut found_snake = false;

        for s in self.snake.segments.iter() {
            if s.distance(head) <= (SEGMENT_RADIUS / 2.0) {
                found_snake = true;
                break;
            }
        }

        if found_snake {
            return CollisionType::Snake;
        }

        let mut found_apple = false;
        let mut to_remove = Vec::new();
        self.apples.iter().enumerate().for_each(|(i, a)| {
            if a.distance(head) <= (SEGMENT_RADIUS + APPLE_RADIUS) {
                found_apple = true;
                to_remove.push(i);
            }
        });

        to_remove.iter().for_each(|i| {
            self.apples.remove(*i);
        });
        if found_apple {
            return CollisionType::Apple;
        } else {
            return CollisionType::No;
        }
    }

    fn calc_next_possition(&self) -> Vec2D {
        let dir = self.snake.dir;
        let head = self.snake.segments.front().unwrap();
        let mut seg = *head + self.snake.dir * SEGMENT_RADIUS;

        const LOWER_BOUND : f64 = 110.0;
        const UPPER_BOUND : f64 = 390.0;

        if seg.x - SEGMENT_RADIUS < LOWER_BOUND && dir.x < 0.0 {
            seg.x = UPPER_BOUND - SEGMENT_RADIUS;
        }
        if seg.x + SEGMENT_RADIUS >= UPPER_BOUND && dir.x > 0.0 {
            seg.x = LOWER_BOUND + SEGMENT_RADIUS;
        }

        if seg.y - SEGMENT_RADIUS < LOWER_BOUND && dir.y < 0.0 {
            seg.y = UPPER_BOUND - SEGMENT_RADIUS;
        }

        if seg.y + SEGMENT_RADIUS >= UPPER_BOUND && dir.y > 0.0 {
            seg.y = LOWER_BOUND + SEGMENT_RADIUS;
        }

        // println!("New Pos: {:?}", seg);

        return seg;
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        self.last_move -= args.dt;
        if self.last_move <= 0.0 {
            self.last_move = UPDATE_TIME;
            let seg = self.calc_next_possition();
            let collision_type = self.check_collisions(&seg);
            match collision_type {
                CollisionType::Apple => {
                    self.new_segments += SEGMENTS_PER_APPLE;
                    self.score += 1;
                },
                CollisionType::Snake => return true,
                _ => ()
            }
            if self.new_segments == 0 {
                self.snake.segments.pop_back();
            } else {
                self.new_segments -= 1;
            }
            self.snake.segments.push_front(seg);
        }

        self.last_spawn -= args.dt;
        if self.last_spawn <= 0.0 {
            self.last_spawn = SPAWN_TIME;
            let mut rng = rand::thread_rng();
            let x = rng.gen_range(100.0, 400.0);
            let y = rng.gen_range(100.0, 400.0);
            self.apples.push(Vec2D::new(x, y));
        }
        return false;
    }
    
}

fn main() {
    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "Omni Snake",
            [500, 500]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App::new(opengl);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(Button::Keyboard(btn)) = e.press_args() {
            match btn {
                Key::Left => app.snake.dir.rotate(&Angle::from_deg(-TURN_ANGLE)),
                Key::Right => app.snake.dir.rotate(&Angle::from_deg(TURN_ANGLE)),
                _ => ()
            }
        }

        if let Some(u) = e.update_args() {
            if app.update(&u) {
                println!("GAME OVER !!!");
                println!("Score: {:?}", app.score);
                break;
            }
        }
    }
}