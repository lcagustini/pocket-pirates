extern crate sdl2;

use std::ops::{Add, Sub, Mul};

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Scancode;
use sdl2::image::LoadTexture;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) =>
        (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

const BG_COLOR: Color = Color{r: 0, g: 0, b: 0, a: 255};

const FONT_SIZE: u16 = 20;

const TILE_WIDTH: isize = 132;
const TILE_HEIGHT: isize = 99;
const TILE_GROUND: isize = 2*TILE_HEIGHT/3;

const HALF_TILE_WIDTH: isize = TILE_WIDTH/2;
const HALF_TILE_HEIGHT: isize = TILE_GROUND/2;

#[derive(Debug, Copy, Clone)]
struct Vector {
    x: f32,
    y: f32,
}
impl Vector {
    fn normalize(&mut self) {
        let ln = ((self.x*self.x + self.y*self.y) as f32).sqrt();
        if ln == 0.0 {
            return;
        }

        let div = 1.0 / ln;
        self.x *= div;
        self.y *= div;
    }

    fn dot(&self, other: Vector) -> f32 {
        self.x*other.x + self.y*other.y
    }
}
impl Add for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector{ x: self.x + other.x, y: self.y + other.y }
    }
}
impl Sub for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector{ x: self.x - other.x, y: self.y - other.y }
    }
}
impl Mul<f32> for Vector {
    type Output = Vector;

    fn mul(self, other: f32) -> Vector {
        Vector{ x: other*self.x, y: other*self.y }
    }
}

struct Object {
    texture_id: usize,
    
    width: usize,
    height: usize,

    x: isize,
    y: isize,

    offset_x: isize,
    offset_y: isize,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let image_context = sdl2::image::init(sdl2::image::INIT_PNG);
    
    let window = video_subsystem.window("Pocket Pirates", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();
    
    let mut font = ttf_context.load_font("roboto.ttf", FONT_SIZE).unwrap();
    font.set_style(sdl2::ttf::STYLE_NORMAL);

    let mut canvas = window.into_canvas().accelerated().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let textures = vec!(
        texture_creator.load_texture("assets/grass.png").unwrap(),
        texture_creator.load_texture("assets/tree.png").unwrap(),
        texture_creator.load_texture("assets/water.png").unwrap(),
        texture_creator.load_texture("assets/sand.png").unwrap(),
    );
    
    let map: [[usize; 20]; 20] = [
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
        [2; 20],
    ];

    let mut objects: Vec<Object> = Vec::new();
    objects.push(Object{texture_id: 1,
        width: 132,
        height: 195,
        x: 7,
        y: 9,
        offset_x: 0,
        offset_y: -150});
    objects.push(Object{texture_id: 1,
        width: 132,
        height: 195,
        x: 7,
        y: 10,
        offset_x: 0,
        offset_y: -150});

    let mut camera = Vector{x: 400.0, y: 0.0};

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        //Event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },

                _ => {}
            }
        }

        {
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::W) {
                camera.y += 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::A) {
                camera.x += 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::S) {
                camera.y -= 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::D) {
                camera.x -= 5.0;
            }
        }

        //Drawing
        canvas.set_draw_color(BG_COLOR);
        canvas.clear();
        {
            let (w_width, w_height) = canvas.window().size();

            for y in 0..map.len() as isize {
                for x in 0..map.len() as isize {
                    let rect = rect!(camera.x as isize + x * HALF_TILE_WIDTH - y * HALF_TILE_WIDTH,
                                     camera.y as isize + x * HALF_TILE_HEIGHT + y * HALF_TILE_HEIGHT,
                                     TILE_WIDTH, TILE_HEIGHT);

                    canvas.copy(&textures[map[x as usize][y as usize]], None, rect).unwrap();
                }
            }

            for obj in &objects {
                let rect = rect!(camera.x as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x,
                                 camera.y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y,
                                 obj.width, obj.height);

                canvas.copy(&textures[obj.texture_id], None, rect).unwrap();
            }
        }
        canvas.present();        
    }
}
