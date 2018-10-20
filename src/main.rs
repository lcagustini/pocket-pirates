extern crate sdl2;

use std::ops::{Add, Sub, Mul};

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::keyboard::Scancode;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;
use sdl2::render::BlendMode;

macro_rules! rect(($x:expr, $y:expr, $w:expr, $h:expr) =>
                  (sdl2::rect::Rect::new($x as i32, $y as i32, $w as u32, $h as u32)));

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

const BG_COLOR: Color = Color{r: 0, g: 0, b: 0, a: 255};
const UI_BG_COLOR: Color = Color{r: 0, g: 0, b: 0, a: 110};
const UI_BUTTON_COLOR: Color = Color{r: 255, g: 150, b: 150, a: 110};

const ACTION_HUD_BORDER: u32 = 5;
const ACTION_HUD_WIDTH: u32 = 700;
const ACTION_HUD_HEIGHT: u32 = 200;
const ACTION_HUD_BUTTON_WIDTH: u32 = 300;
const ACTION_HUD_BUTTON_HEIGHT: u32 = 95;

const FONT_SIZE: u16 = 40;

const TILE_WIDTH: isize = 132;
const TILE_HEIGHT: isize = 99;
const TILE_GROUND: isize = 2*TILE_HEIGHT/3;

const HALF_TILE_WIDTH: isize = TILE_WIDTH/2;
const HALF_TILE_HEIGHT: isize = TILE_GROUND/2;

const BOAT_X: isize = 8;
const BOAT_Y: isize = 12;
const BOAT_OFFSET_X: isize = 0;
const BOAT_OFFSET_Y: isize = 30;
const BOAT_COST: isize = 10;

const BOAT_PLAYER_COMBAT_X: isize = 9;
const BOAT_PLAYER_COMBAT_Y: isize = 12;
const BOAT_ENEMY_COMBAT_X: isize = 10;
const BOAT_ENEMY_COMBAT_Y: isize = 4;

const CAMERA_X: isize = 500;
const CAMERA_Y: isize = -400;

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

#[derive (Copy, Clone)]
struct Object {
    texture_id: usize,

    x: isize,
    y: isize,

    offset_x: isize,
    offset_y: isize,
}

struct Boat {
    health: isize,

    wood: isize,
    mineral: isize,

    obj: Option<Object>,
}

fn gather_resource(player_id : &mut usize, player_boat : &mut Boat, objects : &mut Vec<Object>, texture_id : usize) {
    let mut x = 0;
    let mut y = 0;
    match texture_id {
        4 => {
            x = objects[*player_id].x;
            y = objects[*player_id].y-1;
        },
        5 => {
            x = objects[*player_id].x-1;
            y = objects[*player_id].y;
        },
        6 => {
            x = objects[*player_id].x;
            y = objects[*player_id].y+1;
        },
        7 => {
            x = objects[*player_id].x+1;
            y = objects[*player_id].y;
        },

        _ => ()
    }

    let mut target : isize = -1;
    for (i, obj) in objects.iter().enumerate() {
        if obj.x == x && obj.y == y {
            if obj.texture_id == 1 { // tree
                target = i as isize;
            }
        }
    }
    if target != -1 {
        if *player_id as isize > target {
            *player_id -= 1;
        }
        objects.remove(target as usize);
        player_boat.wood += 5;
    }
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
    //canvas.set_logical_size(1920, 1080);
    //canvas.set_scale(0.5, 0.5);

    let textures = vec!(
        texture_creator.load_texture("assets/grass.png").unwrap(),
        texture_creator.load_texture("assets/tree.png").unwrap(),
        texture_creator.load_texture("assets/water.png").unwrap(),
        texture_creator.load_texture("assets/sand.png").unwrap(),
        texture_creator.load_texture("assets/player_NE.png").unwrap(),
        texture_creator.load_texture("assets/player_NW.png").unwrap(),
        texture_creator.load_texture("assets/player_SW.png").unwrap(),
        texture_creator.load_texture("assets/player_SE.png").unwrap(),
        texture_creator.load_texture("assets/wood.png").unwrap(),
        texture_creator.load_texture("assets/mineral.png").unwrap(),
        texture_creator.load_texture("assets/boat_small_NE.png").unwrap(),
        texture_creator.load_texture("assets/boat_small_NW.png").unwrap(),
        texture_creator.load_texture("assets/boat_small_SW.png").unwrap(),
        texture_creator.load_texture("assets/boat_small_SE.png").unwrap(),
    );

    let map: [[usize; 30]; 30] = [
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 3, 0, 0, 0, 0, 0, 0, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30],
        [2; 30]];

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

    let mut player_boat = Boat{health: 20, wood: 0, mineral: 0, obj: None};

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        //Event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },

                Event::KeyUp { keycode: Some(Keycode::E), .. } => {
                    let tid = objects[player_id].texture_id;
                    gather_resource(&mut player_id, &mut player_boat, &mut objects, tid);

                    if (objects[player_id].x - BOAT_X).abs() <= 1 && (objects[player_id].y - BOAT_Y).abs() <= 1 && player_boat.obj.is_some() {
                        start_combat_phase(player_boat, canvas, textures, font, event_pump);
                        break 'running
                    }
                },

                Event::KeyUp { keycode: Some(Keycode::B), .. } => {
                    if player_boat.wood >= BOAT_COST {
                        player_boat.wood -= BOAT_COST;
                        player_boat.obj = Some(Object{texture_id: 13, x: BOAT_X, y: BOAT_Y, offset_x: BOAT_OFFSET_X, offset_y: BOAT_OFFSET_Y});
                    }
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
        }

        //Drawing
        canvas.set_draw_color(BG_COLOR);
        canvas.clear();
        let (w_width, w_height) = canvas.window().size();
        {

            for y in 0..map.len() as isize {
                for x in 0..map.len() as isize {
                    let rect = rect!(CAMERA_X as isize + x * HALF_TILE_WIDTH - y * HALF_TILE_WIDTH,
                                     CAMERA_Y as isize + x * HALF_TILE_HEIGHT + y * HALF_TILE_HEIGHT,
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

                let x = offset.0 + CAMERA_X as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x;
                let y = offset.1 + CAMERA_Y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y;

                let rect = rect!(x, y, texture_info.width, texture_info.height);

                canvas.copy(texture, None, rect).unwrap();
            }
        }

        // draw boat
        {
            match player_boat.obj {
                Some(obj) => {
                    let texture = &textures[obj.texture_id];
                    let texture_info = texture.query();
                    let x = CAMERA_X as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x;
                    let y = CAMERA_Y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y;
                    let rect = rect!(x, y, texture_info.width, texture_info.height);
                    canvas.copy(texture, None, rect).unwrap();
                }, 
                None => ()
            }
        }

        // draw materials HUD
        // TODO: maybe not rerender every frame
        {
            let rect = rect!(w_width - 125, w_height - (2 * FONT_SIZE as u32 + 10), 120, 2 * FONT_SIZE as u32 + 5);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BG_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let font_s = font.render(&player_boat.wood.to_string()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let font_t = texture_creator.create_texture_from_surface(&font_s).unwrap();
            let font_t_info = font_t.query();
            let rect = rect!(w_width - font_t_info.width - 5, w_height - font_t_info.height * 2 - 5, font_t_info.width, font_t_info.height);
            canvas.copy(&font_t, None, rect).unwrap();

            let font_s = font.render(&player_boat.mineral.to_string()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let font_t = texture_creator.create_texture_from_surface(&font_s).unwrap();
            let font_t_info = font_t.query();
            let rect = rect!(w_width - font_t_info.width - 5, w_height - font_t_info.height - 5, font_t_info.width, font_t_info.height);
            canvas.copy(&font_t, None, rect).unwrap();

            let metal_texture = &textures[9];
            let tex_info = metal_texture.query();
            let rect = rect!(w_width - 120, w_height - (font_t_info.height) - 5, tex_info.width as f32 * (font_t_info.height as f32 / tex_info.height as f32), font_t_info.height);
            canvas.copy(&metal_texture, None, rect).unwrap();

            let wood_texture = &textures[8];
            let tex_info = wood_texture.query();
            let rect = rect!(w_width - 120, w_height - (font_t_info.height * 2) - 5, tex_info.width as f32 * (font_t_info.height as f32 / tex_info.height as f32), font_t_info.height);
            canvas.copy(&wood_texture, None, rect).unwrap();
        }
        
        if player_timer > 0 {
            player_timer -= 1;
        }

        canvas.present();
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

fn start_combat_phase(mut player_boat : Boat, mut canvas : sdl2::render::Canvas<sdl2::video::Window>, textures : Vec<sdl2::render::Texture>, font : sdl2::ttf::Font, mut event_pump : sdl2::EventPump) {
    let texture_creator = canvas.texture_creator();
    let map: [[usize; 30]; 30] = [[2; 30]; 30];

    let mut enemy_boat = Boat {health: 20, wood: 15, mineral: 5,
        obj: Some(Object{texture_id: 11, x: BOAT_ENEMY_COMBAT_X, y: BOAT_ENEMY_COMBAT_Y, offset_x: BOAT_OFFSET_X, offset_y: BOAT_OFFSET_Y})};

    player_boat.obj.as_mut().unwrap().x = BOAT_PLAYER_COMBAT_X;
    player_boat.obj.as_mut().unwrap().y = BOAT_PLAYER_COMBAT_Y;

    'running: loop {
        let (w_width, w_height) = canvas.window().size();

        //Event handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },

                _ => {}
            }
        }

        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        for y in 0..map.len() as isize {
            for x in 0..map.len() as isize {
                let rect = rect!(CAMERA_X as isize + x * HALF_TILE_WIDTH - y * HALF_TILE_WIDTH,
                                 CAMERA_Y as isize + x * HALF_TILE_HEIGHT + y * HALF_TILE_HEIGHT,
                                 TILE_WIDTH, TILE_HEIGHT);

                canvas.copy(&textures[map[x as usize][y as usize]], None, rect).unwrap();
            }
        }

        // draw boats
        {
            // player boat
            {
                let obj = player_boat.obj.unwrap();
                let texture = &textures[obj.texture_id];
                let texture_info = texture.query();
                let x = CAMERA_X as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x;
                let y = CAMERA_Y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y;
                let rect = rect!(x, y, texture_info.width, texture_info.height);
                canvas.copy(texture, None, rect).unwrap();
            }

            // enemy boat
            {
                let obj = enemy_boat.obj.unwrap();
                let texture = &textures[obj.texture_id];
                let texture_info = texture.query();
                let x = CAMERA_X as isize + obj.x * HALF_TILE_WIDTH - obj.y * HALF_TILE_WIDTH + obj.offset_x;
                let y = CAMERA_Y as isize + obj.x * HALF_TILE_HEIGHT + obj.y * HALF_TILE_HEIGHT + obj.offset_y;
                let rect = rect!(x, y, texture_info.width, texture_info.height);
                canvas.copy(texture, None, rect).unwrap();
            }
        }

        // draw actions HUD
        {
            let rect = rect!(ACTION_HUD_BORDER, w_height - ACTION_HUD_HEIGHT - ACTION_HUD_BORDER, ACTION_HUD_WIDTH, ACTION_HUD_HEIGHT);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BG_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let rect = rect!(ACTION_HUD_BORDER * 2, w_height - ACTION_HUD_HEIGHT, ACTION_HUD_BUTTON_WIDTH, ACTION_HUD_BUTTON_HEIGHT);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BUTTON_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let rect = rect!(ACTION_HUD_BORDER * 3 + ACTION_HUD_BUTTON_WIDTH, w_height - ACTION_HUD_HEIGHT, ACTION_HUD_BUTTON_WIDTH, ACTION_HUD_BUTTON_HEIGHT);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BUTTON_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let rect = rect!(ACTION_HUD_BORDER * 2, w_height - ACTION_HUD_HEIGHT + ACTION_HUD_BORDER + ACTION_HUD_BUTTON_HEIGHT, ACTION_HUD_BUTTON_WIDTH, ACTION_HUD_BUTTON_HEIGHT);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BUTTON_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let rect = rect!(ACTION_HUD_BORDER * 3 + ACTION_HUD_BUTTON_WIDTH, w_height - ACTION_HUD_HEIGHT + ACTION_HUD_BORDER + ACTION_HUD_BUTTON_HEIGHT,
                             ACTION_HUD_BUTTON_WIDTH, ACTION_HUD_BUTTON_HEIGHT);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BUTTON_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);
        }

        // draw materials HUD
        // TODO: maybe not rerender every frame
        {
            let rect = rect!(w_width - 125, w_height - (2 * FONT_SIZE as u32 + 10), 120, 2 * FONT_SIZE as u32 + 5);
            canvas.set_blend_mode(BlendMode::Blend);
            canvas.set_draw_color(UI_BG_COLOR);
            canvas.fill_rect(rect).unwrap();
            canvas.set_blend_mode(BlendMode::None);

            let font_s = font.render(&player_boat.wood.to_string()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let font_t = texture_creator.create_texture_from_surface(&font_s).unwrap();
            let font_t_info = font_t.query();
            let rect = rect!(w_width - font_t_info.width - 5, w_height - font_t_info.height * 2 - 5, font_t_info.width, font_t_info.height);
            canvas.copy(&font_t, None, rect).unwrap();

            let font_s = font.render(&player_boat.mineral.to_string()).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
            let font_t = texture_creator.create_texture_from_surface(&font_s).unwrap();
            let font_t_info = font_t.query();
            let rect = rect!(w_width - font_t_info.width - 5, w_height - font_t_info.height - 5, font_t_info.width, font_t_info.height);
            canvas.copy(&font_t, None, rect).unwrap();

            let metal_texture = &textures[9];
            let tex_info = metal_texture.query();
            let rect = rect!(w_width - 120, w_height - (font_t_info.height) - 5, tex_info.width as f32 * (font_t_info.height as f32 / tex_info.height as f32), font_t_info.height);
            canvas.copy(&metal_texture, None, rect).unwrap();

            let wood_texture = &textures[8];
            let tex_info = wood_texture.query();
            let rect = rect!(w_width - 120, w_height - (font_t_info.height * 2) - 5, tex_info.width as f32 * (font_t_info.height as f32 / tex_info.height as f32), font_t_info.height);
            canvas.copy(&wood_texture, None, rect).unwrap();
        }

        canvas.present()
    }
}
