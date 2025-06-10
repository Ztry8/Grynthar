use macroquad::{
    camera::{set_camera, Camera2D},
    input::{
        is_mouse_button_down, is_mouse_button_pressed, mouse_delta_position, mouse_position,
        mouse_wheel, MouseButton,
    },
    math::{vec2, IVec2, Vec2},
};

use crate::{
    bot::Bot,
    config::{gen_game, INI_GAME_ERROR, SENSITIVITY_FIELD_NAME},
    level::Level,
    manager::Manager,
    menu::Menu,
    player::Player,
    structs::{Difficulty, Team, UnitType},
};

pub const DEFAULT_CAMERA_ZOOM: u8 = 7;
const MAX_CAMERA_ZOOM: u8 = 8;
const MIN_CAMERA_ZOOM: u8 = 7;

#[derive(PartialEq)]
pub enum GameResult {
    Lose(u32),
    Win(u32),
}

pub struct Game {
    player_attacked: Vec<(IVec2, i16)>,
    difficulty: Difficulty,
    bot: Option<Bot>,
    sensitivity: f32,
    timer_enemy: f32,
    enemy_time: f32,
    player: Player,
    timer_win: f32,
    win_time: f32,
    level: Level,
    zoom: u8,
}

impl Game {
    pub fn new(units: Vec<Vec<UnitType>>, difficulty: Difficulty) -> Self {
        let mut config = gen_game();
        let config = config.with_general_section();
        let mut level = Level::new();
        level.capture(&IVec2::ZERO, &Team::Player);

        Self {
            win_time: if difficulty == Difficulty::Normal {
                30.0
            } else {
                60.0
            },
            enemy_time: if difficulty == Difficulty::Normal {
                40.0
            } else {
                20.0
            },
            player_attacked: Vec::new(),
            sensitivity: config
                .get(SENSITIVITY_FIELD_NAME)
                .expect(INI_GAME_ERROR)
                .parse::<f32>()
                .expect(INI_GAME_ERROR),
            zoom: DEFAULT_CAMERA_ZOOM,
            player: Player::new(units),
            timer_enemy: 0.0,
            timer_win: 0.0,
            difficulty,
            bot: None,
            level,
        }
    }

    pub fn update(
        &mut self,
        manager: &Manager,
        menu: &Menu,
        camera: &mut Camera2D,
        zoom: Vec2,
        delta: f32,
    ) -> Option<GameResult> {
        let mouse = mouse_position();

        let mut closed = false;
        if is_mouse_button_down(MouseButton::Middle) {
            let mouse = mouse_delta_position();
            camera.target += mouse * self.sensitivity;
            closed = true;
        }

        if is_mouse_button_down(MouseButton::Right) {
            camera.target = vec2(0.0, 0.0);
        }

        let wheel = mouse_wheel().1;
        if wheel < 0.0 && self.zoom > MIN_CAMERA_ZOOM {
            self.zoom -= 1;
            camera.zoom -= zoom;
        } else if wheel > 0.0 && self.zoom < MAX_CAMERA_ZOOM {
            self.zoom += 1;
            camera.zoom += zoom;
        }

        set_camera(camera);
        self.level.update(camera);

        let result = self.player.update(
            menu,
            manager,
            &self.player_attacked,
            if let Some(bot) = &self.bot {
                bot.positions()
            } else {
                Vec::new()
            },
            &mut self.level,
            Level::hex(camera.screen_to_world(vec2(mouse.0, mouse.1))),
            is_mouse_button_pressed(MouseButton::Left),
            delta,
        );

        if let Some(bot) = &mut self.bot {
            self.player_attacked = bot.update(manager, result, &mut self.level, delta);
        } else {
            self.timer_enemy += delta;
            if self.timer_enemy >= self.enemy_time {
                self.timer_enemy = 0.0;
                self.bot = Some(Bot::new(&mut self.level, self.difficulty.clone()));
            }
        }

        let control = self.level.control_player();
        self.player.draw_ui(manager, closed, control);

        if self.bot.is_none() {
            manager.draw_text(
                false,
                &format!(
                    "{} {} {}",
                    manager.get_text(5),
                    (self.enemy_time - self.timer_enemy).floor() as u8 + 1,
                    manager.get_text(4)
                ),
                1.1,
                1.0,
            );
        }

        if control >= 100 && self.bot.is_some() {
            manager.draw_text(
                false,
                &format!(
                    "{} {} {}",
                    manager.get_text(3),
                    (self.win_time - self.timer_win).floor() as u8 + 1,
                    manager.get_text(4)
                ),
                1.1,
                1.0,
            );

            self.timer_win += delta;
            if self.timer_win >= self.win_time {
                return Some(GameResult::Win(self.player.fine));
            }
        } else {
            self.timer_win = 0.0;
        }

        if let Some(bot) = &self.bot {
            if bot.lose() {
                self.bot = None;
            }
        }

        if self.player.lose() {
            Some(GameResult::Lose(self.player.fine))
        } else {
            None
        }
    }
}
