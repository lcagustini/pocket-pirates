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

    x: isize,
    y: isize,

    offset_x: isize,
    offset_y: isize,
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::INIT_PNG);

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
        texture_creator.load_texture("assets/player_NE.png").unwrap(),
        texture_creator.load_texture("assets/player_NW.png").unwrap(),
        texture_creator.load_texture("assets/player_SW.png").unwrap(),
        texture_creator.load_texture("assets/player_SE.png").unwrap(),
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
        [2; 20]];

    let mut objects: Vec<Object> = vec!(
        Object{texture_id: 4,
            x: 8,
            y: 10,
            offset_x: 35,
            offset_y: -60},
        Object{texture_id: 1,
            x: 7,
            y: 9,
            offset_x: 0,
            offset_y: -150},
        Object{texture_id: 1,
            x: 7,
            y: 10,
            offset_x: 0,
            offset_y: -150});
    let mut player_id = 0;
    let mut player_timer = 0;
    let mut player_last_pos = (0, 0);

    let mut camera = Vector{x: 0.0, y: 0.0};

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
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::W) && player_timer == 0 {
                player_last_pos = (objects[player_id].x, objects[player_id].y);
                objects[player_id].texture_id = 4;

                let mut can_walk = true;
                for obj in &objects {
                    if obj.x == player_last_pos.0 && obj.y == player_last_pos.1 - 1 {
                        can_walk = false;
                        break;
                    }
                }
                if map[player_last_pos.0 as usize][player_last_pos.1 as usize -1] == 2 {
                    can_walk = false;
                }
                if can_walk {
                    objects[player_id].y -= 1;
                    player_timer = 20;
                }
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::A) && player_timer == 0 {
                player_last_pos = (objects[player_id].x, objects[player_id].y);
                objects[player_id].texture_id = 5;

                let mut can_walk = true;
                for obj in &objects {
                    if obj.x == player_last_pos.0 - 1 && obj.y == player_last_pos.1 {
                        can_walk = false;
                        break;
                    }
                }
                if map[player_last_pos.0 as usize -1][player_last_pos.1 as usize] == 2 {
                    can_walk = false;
                }
                if can_walk {
                    objects[player_id].x -= 1;
                    player_timer = 20;
                }
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::S) && player_timer == 0 {
                player_last_pos = (objects[player_id].x, objects[player_id].y);
                objects[player_id].texture_id = 6;

                let mut can_walk = true;
                for obj in &objects {
                    if obj.x == player_last_pos.0 && obj.y == player_last_pos.1 + 1 {
                        can_walk = false;
                        break;
                    }
                }
                if map[player_last_pos.0 as usize][player_last_pos.1 as usize +1] == 2 {
                    can_walk = false;
                }
                if can_walk {
                    objects[player_id].y += 1;
                    player_timer = 20;
                }
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::D) && player_timer == 0 {
                player_last_pos = (objects[player_id].x, objects[player_id].y);
                objects[player_id].texture_id = 7;

                let mut can_walk = true;
                for obj in &objects {
                    if obj.x == player_last_pos.0 + 1 && obj.y == player_last_pos.1 {
                        can_walk = false;
                        break;
                    }
                }
                if map[player_last_pos.0 as usize +1][player_last_pos.1 as usize] == 2 {
                    can_walk = false;
                }
                if can_walk {
                    objects[player_id].x += 1;
                    player_timer = 20;
                }
            }

            if event_pump.keyboard_state().is_scancode_pressed(Scancode::Up) {
                camera.y += 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::Left) {
                camera.x += 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::Down) {
                camera.y -= 5.0;
            }
            if event_pump.keyboard_state().is_scancode_pressed(Scancode::Right) {
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

            bubble_sort(&mut objects, &mut player_id);
            for (i, obj) in objects.iter().enumerate() {
                let texture = &textures[obj.texture_id];
                let texture_info = texture.query();

                let mut offset = (0, 0);
                if i == player_id {
                    let dx = obj.x - player_last_pos.0;
                    let dy = obj.y - player_last_pos.1;

                    let ratio = player_timer as f32 / 20.0;

                    if dx == -1 {
                        offset = ((ratio * HALF_TILE_WIDTH as f32) as isize, (ratio * HALF_TILE_HEIGHT as f32) as isize);
                    }
                    if dx == 1 {
                        offset = (-(ratio * HALF_TILE_WIDTH as f32) as isize, -(ratio * HALF_TILE_HEIGHT as f32) as isize);
                    }
                    if dy == -1 {
                        offset = (-(ratio * HALF_TILE_WIDTH as f32) as isize, (ratio * HALF_TILE_HEIGHT as f32) as isize);
                    }
                    if dy == 1 {
                        offset = ((ratio * HALF_TILE_WIDTH as f32) as isize, -(ratio * HALF_TILE_HEIGHT as f32) as isize);
                    }
                }

                let x = offset.0 + camera.x as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x;
                let y = offset.1 + camera.y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y;

                let rect = rect!(x, y, texture_info.width, texture_info.height);

                canvas.copy(texture, None, rect).unwrap();
            }
        }
        canvas.present();

        if player_timer > 0 {
            player_timer -= 1;
        }
    }
}

fn bubble_sort(obj: &mut Vec<Object>, player_id: &mut usize) {
    for i in 0..obj.len() {
        for j in i+1..obj.len() {
            if obj[i].y > obj[j].y || (obj[i].y == obj[j].y && obj[i].x > obj[j].x) {
                if i == *player_id {
                    *player_id = j;
                }
                else if j == *player_id {
                    *player_id = i;
                }
                obj.swap(i, j);
            }
        }
    }
}
