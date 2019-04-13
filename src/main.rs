extern crate math2d;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use math2d::math::common::*;
use math2d::math::vector::Vec2D;
use math2d::math::angle::Angle;

use piston::window::{WindowSettings, Window};
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use std::collections::VecDeque;
use rand::Rng;

const SEGMENT_RADIUS: f64 = 5.0;            // [Pix]
const APPLE_RADIUS: f64 = 5.0;              // [Pix]
const UPDATE_TIME: f64 = 0.05;              // [S]
const SPAWN_TIME: f64 = 5.0;                // [S]
const SEGMENTS_PER_APPLE: usize = 10;       // [#]
const TURN_ANGLE: f64 = 20.0;               // [Deg]

const WINDOW_SIZE: (u32, u32) = (1000, 1000);
// const WALL_SIZE: (f64, f64) = (WINDOW_SIZE.0 as f64 * 0.22, WINDOW_SIZE.1 as f64 * 0.22);
const WALL_SIZE: (f64, f64) = (1.0, 1.0);
const BOUNDS_X: (f64, f64) = (WALL_SIZE.0, (WINDOW_SIZE.0 as f64) - WALL_SIZE.0);
const BOUNDS_Y: (f64, f64) = (WALL_SIZE.1, (WINDOW_SIZE.1 as f64) - WALL_SIZE.1);

struct Snake {
    dir: Vec2D,
    segments: VecDeque<Vec2D>,
}

impl Snake {
    fn new(start_pos: Vec2D, start_dir: Vec2D) -> Snake {
        let seg_separation = SEGMENT_RADIUS * 2.0;
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
            snake: Snake::new(
                Vec2D::new(WINDOW_SIZE.0 as f64 / 2.0, WINDOW_SIZE.1 as f64 / 2.0), 
                Vec2D::new(-1.0, 0.0)),
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
        let corner_graphics = rectangle::Rectangle::new(RED);

        let snake = &self.snake;
        let apples = &self.apples;

        self.gl.draw(args.viewport(), |c, gl| {
            clear(BLACK, gl);
            
            let corners = vec!(
                c.transform.trans(BOUNDS_X.0, BOUNDS_Y.0),
                c.transform.trans(BOUNDS_X.1, BOUNDS_Y.0),
                c.transform.trans(BOUNDS_X.1, BOUNDS_Y.1),
                c.transform.trans(BOUNDS_X.0, BOUNDS_Y.1)
            );
            corners.iter().for_each(|tr|{
                corner_graphics.draw([0.0, 0.0, 2.0, 2.0], &c.draw_state, *tr, gl);
            });

            apples.iter().for_each(|a| {
                let (x, y) = a.as_tuple();
                let transform = c.transform.trans(x, y)
                    .trans(-APPLE_RADIUS, -APPLE_RADIUS);
                apple_graphics.draw(ellipse::circle(APPLE_RADIUS, APPLE_RADIUS, APPLE_RADIUS), &c.draw_state, transform, gl);
            });
            
            snake.segments.iter().enumerate().for_each(|(i, s)|{
                let (x, y) = s.as_tuple();
                let rad = if i == 0 {SEGMENT_RADIUS + 1.0} else {SEGMENT_RADIUS};
                let transform = c.transform.trans(x, y).trans(-rad, -rad);
                segment_graphics.draw(ellipse::circle(rad, rad, rad), &c.draw_state, transform, gl);
            });
        });
    }

    fn check_collisions(&mut self, head: &Vec2D) -> CollisionType {
        let mut found_snake = false;

        for s in self.snake.segments.iter() {
            if s.distance(head) <= (SEGMENT_RADIUS) {
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
        let mut seg = *head + self.snake.dir * SEGMENT_RADIUS * 2.0;

        if seg.x - SEGMENT_RADIUS < BOUNDS_X.0 && dir.x < 0.0 {
            seg.x = BOUNDS_X.1 - SEGMENT_RADIUS;
        }
        if seg.x + SEGMENT_RADIUS >= BOUNDS_X.1 && dir.x > 0.0 {
            seg.x = BOUNDS_X.0 + SEGMENT_RADIUS;
        }

        if seg.y - SEGMENT_RADIUS < BOUNDS_Y.0 && dir.y < 0.0 {
            seg.y = BOUNDS_Y.1 - SEGMENT_RADIUS;
        }

        if seg.y + SEGMENT_RADIUS >= BOUNDS_Y.1 && dir.y > 0.0 {
            seg.y = BOUNDS_Y.0 + SEGMENT_RADIUS;
        }

        return seg;
    }

    fn update(&mut self, args: &UpdateArgs) -> bool {
        // return false;
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
            let x = rng.gen_range(BOUNDS_X.0, BOUNDS_X.1);
            let y = rng.gen_range(BOUNDS_Y.0, BOUNDS_Y.1);
            self.apples.push(Vec2D::new(x, y));
        }
        return false;
    }
    
}

fn main() {
    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let mut window: GlutinWindow = WindowSettings::new(
            "Omni Snake",
            [WINDOW_SIZE.0, WINDOW_SIZE.1]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

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