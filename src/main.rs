#![windows_subsystem = "windows"]
use std::io::Cursor;

use config::{
    gen_graphics, ANTIALIASING_FIELD_NAME, FULLSCREEN_FIELD_NAME, HIGHDPI_FIELD_NAME,
    INI_GRAPHICS_ERROR,
};
use game::{Game, GameResult, DEFAULT_CAMERA_ZOOM};
use image::ImageReader;
use macroquad::prelude::*;
use manager::Manager;
use menu::Menu;
use miniquad::conf::Icon;
use structs::{Difficulty, FOG_COLOR};

mod bot;
mod config;
mod game;
mod level;
mod manager;
mod menu;
mod player;
mod squad;
mod structs;
mod unit;

const WIDTH_TILES: f32 = 400.0;
const HEIGHT_TILES: f32 = 225.0;
const TILE_SIZE: f32 = 2.0;

struct Instance {
    pub game: Game,
    pub zoom: Vec2,
    pub camera: Camera2D,
}

enum GameState {
    Play(Box<Instance>),
    Init(Difficulty),
    Menu(GameResult),
}

fn config() -> Conf {
    #[cfg(all(not(debug_assertions), not(target_arch = "wasm32")))]
    {
        use native_dialog::{MessageDialog, MessageType};
        use std::{panic, process::abort};
        panic::set_hook(Box::new(|error| {
            MessageDialog::new()
                .set_title("Engine Error!")
                .set_type(MessageType::Error)
                .set_text(
                    format!(
                        "An error has occurred: {}. The game files are corrupted!",
                        if let Some(msg) = error.payload().downcast_ref::<&str>() {
                            msg
                        } else if let Some(msg) = error.payload().downcast_ref::<String>() {
                            msg
                        } else {
                            "unknown"
                        }
                    )
                    .as_str(),
                )
                .show_alert()
                .unwrap_or_else(|_| abort());
            abort();
        }));
    }

    let mut config = gen_graphics();
    let config = config.with_general_section();
    let scale = config
        .get("Scale")
        .expect(INI_GRAPHICS_ERROR)
        .parse::<i32>()
        .expect(INI_GRAPHICS_ERROR) as f32
        * TILE_SIZE;

    Conf {
        window_title: format!(
            "Grynthar {} by {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS")
        ),
        window_width: (WIDTH_TILES * scale) as i32,
        window_height: (HEIGHT_TILES * scale) as i32,
        high_dpi: config
            .get(HIGHDPI_FIELD_NAME)
            .expect(INI_GRAPHICS_ERROR)
            .parse::<bool>()
            .expect(INI_GRAPHICS_ERROR),
        fullscreen: config
            .get(FULLSCREEN_FIELD_NAME)
            .expect(INI_GRAPHICS_ERROR)
            .parse::<bool>()
            .expect(INI_GRAPHICS_ERROR),
        sample_count: config
            .get(ANTIALIASING_FIELD_NAME)
            .expect(INI_GRAPHICS_ERROR)
            .parse::<i32>()
            .expect(INI_GRAPHICS_ERROR),
        window_resizable: false,
        icon: Some(Icon {
            small: ImageReader::new(Cursor::new(include_bytes!("../assets/logo/16.png")))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8()
                .into_raw()
                .try_into()
                .unwrap(),
            medium: ImageReader::new(Cursor::new(include_bytes!("../assets/logo/32.png")))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8()
                .into_raw()
                .try_into()
                .unwrap(),
            big: ImageReader::new(Cursor::new(include_bytes!("../assets/logo/64.png")))
                .with_guessed_format()
                .unwrap()
                .decode()
                .unwrap()
                .to_rgba8()
                .into_raw()
                .try_into()
                .unwrap(),
        }),
        ..Default::default()
    }
}

pub fn seed() -> u64 {
    // use std::time::{SystemTime, UNIX_EPOCH};
    // match SystemTime::now().duration_since(UNIX_EPOCH) {
    //     Ok(n) => n,
    //     Err(n) => n.duration(),
    // }
    // .as_secs()
    0
}

#[macroquad::main(config)]
async fn main() {
    let mut state = GameState::Menu(GameResult::Lose(0));
    let mut manager = Manager::new("English", screen_width() / WIDTH_TILES / TILE_SIZE).await;
    let mut menu = Menu::new(&manager);

    loop {
        clear_background(FOG_COLOR);

        menu.game_timer += get_frame_time() as f64;
        state = match state {
            GameState::Menu(result) => {
                if let Some(difficulty) = menu.update(&mut manager, &result) {
                    manager.stop_music();
                    GameState::Init(difficulty)
                } else {
                    GameState::Menu(result)
                }
            }
            GameState::Play(mut instance) => {
                if let Some(result) = instance.game.update(
                    &manager,
                    &menu,
                    &mut instance.camera,
                    instance.zoom,
                    get_frame_time(),
                ) {
                    manager.start_music();
                    GameState::Menu(result)
                } else {
                    GameState::Play(instance)
                }
            }
            GameState::Init(difficulty) => {
                let mut camera = Camera2D::from_display_rect(Rect::new(
                    0.0,
                    0.0,
                    TILE_SIZE * WIDTH_TILES * 2.0,
                    TILE_SIZE * HEIGHT_TILES * 2.0,
                ));

                camera.target = vec2(0.0, 0.0);
                camera.zoom.y *= -1.0;
                let zoom = camera.zoom;
                camera.zoom += zoom * DEFAULT_CAMERA_ZOOM as f32;

                GameState::Play(Box::new(Instance {
                    zoom,
                    camera,
                    game: Game::new(menu.get_units(), difficulty),
                }))
            }
        };

        next_frame().await
    }
}
