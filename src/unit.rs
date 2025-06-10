use std::f32::consts::PI;

use macroquad::{
    math::{vec2, IVec2, Vec2},
    shapes::{draw_circle, draw_circle_lines, draw_line},
};

use crate::{
    level::Level,
    structs::{
        Team, UnitType, BORDER, ENEMY_BORDER_COLOR, ENEMY_COLOR, PLAYER_BORDER_COLOR, PLAYER_COLOR,
        SNIPER_ENEMY_BORDER_COLOR, SNIPER_ENEMY_COLOR, SNIPER_PLAYER_BORDER_COLOR,
        SNIPER_PLAYER_COLOR,
    },
    TILE_SIZE,
};

pub const UNIT_SIZE: f32 = TILE_SIZE * 0.5;
const MAX_TIMER: f32 = 5.0;
const LINE_THICK: f32 = BORDER;
const LINE_LENGTH: f32 = TILE_SIZE;
const SCOUT_SIZE: f32 = TILE_SIZE * 0.3;

#[derive(PartialEq, Clone)]
pub struct Unit {
    pub r#type: UnitType,
    pub start_pos: IVec2,
    pub end_pos: IVec2,
    pub health: i16,
    max_health: i16,
    elapsed: f32,
    tpos: Vec2,
    speed: f32,
    angle: f32,
    timer: f32,
}

impl Unit {
    pub fn new(r#type: &UnitType, pos: IVec2, speed: f32) -> Self {
        let health = match r#type {
            UnitType::Infantry => 100,
            UnitType::Engineer => 120,
            UnitType::Turret => 50,
            UnitType::Scout => 75,
            UnitType::Medic => 90,
            UnitType::Sniper => 80,
        };

        Self {
            tpos: vec2(pos.x as f32, pos.y as f32),
            max_health: health,
            timer: MAX_TIMER,
            start_pos: pos,
            end_pos: pos,
            elapsed: 0.0,
            angle: 0.0,
            r#type: r#type.clone(),
            health,
            speed,
        }
    }

    pub fn heal(&mut self) {
        self.health = self.max_health.min(self.health * 3 / 2);
    }

    pub fn fire(&mut self, target: Option<IVec2>, delta: f32) -> bool {
        if self.r#type != UnitType::Engineer {
            if let Some(target) = target {
                self.angle = (self.start_pos.y as f32 - target.y as f32)
                    .atan2(self.start_pos.x as f32 - target.x as f32)
                    + PI;
            }

            if self.timer
                >= match self.r#type {
                    UnitType::Infantry => 1.0,
                    UnitType::Engineer => 1.2,
                    UnitType::Scout | UnitType::Turret => 0.9,
                    UnitType::Medic => 1.5,
                    UnitType::Sniper => 3.0,
                }
            {
                return true;
            } else {
                self.timer += delta;
            }
        }

        false
    }

    pub fn zero_timer(&mut self) {
        self.timer = 0.0;
    }

    pub fn update(&mut self, delta: f32) -> bool {
        self.elapsed += delta;
        let t = (self.elapsed / self.speed).min(1.0);
        self.tpos = self.start_pos.as_vec2() + (self.end_pos - self.start_pos).as_vec2() * t;

        if t >= 1.0 {
            self.start_pos = self.end_pos;
            self.elapsed = 0.0;
            return true;
        }

        false
    }

    pub fn render(&self, team: &Team, active: bool) {
        let (x, y) = Level::convert(self.tpos.x, self.tpos.y);
        let sniper = self.r#type == UnitType::Sniper;
        let turret = self.r#type == UnitType::Turret;

        draw_circle(
            x,
            y,
            UNIT_SIZE,
            match team {
                Team::Player => {
                    if sniper {
                        SNIPER_PLAYER_COLOR
                    } else if turret {
                        PLAYER_BORDER_COLOR
                    } else {
                        PLAYER_COLOR
                    }
                }
                Team::Computer => {
                    if sniper {
                        SNIPER_ENEMY_COLOR
                    } else if turret {
                        ENEMY_BORDER_COLOR
                    } else {
                        ENEMY_COLOR
                    }
                }
            },
        );

        let color = match team {
            Team::Player => {
                if sniper {
                    SNIPER_PLAYER_BORDER_COLOR
                } else if turret {
                    PLAYER_COLOR
                } else {
                    PLAYER_BORDER_COLOR
                }
            }
            Team::Computer => {
                if sniper {
                    SNIPER_ENEMY_BORDER_COLOR
                } else if turret {
                    ENEMY_COLOR
                } else {
                    ENEMY_BORDER_COLOR
                }
            }
        };

        if self.r#type != UnitType::Engineer {
            draw_line(
                x,
                y,
                x + self.angle.cos() * LINE_LENGTH,
                y + self.angle.sin() * LINE_LENGTH,
                LINE_THICK,
                color,
            );

            if self.r#type == UnitType::Infantry {
                draw_circle(x, y, SCOUT_SIZE, color);
            } else if self.r#type == UnitType::Medic {
                draw_line(x, y - UNIT_SIZE, x, y + UNIT_SIZE, LINE_THICK, color);
                draw_line(x - UNIT_SIZE, y, x + UNIT_SIZE, y, LINE_THICK, color);
            }
        }

        if self.r#type != UnitType::Turret && active {
            draw_circle_lines(x, y, UNIT_SIZE, LINE_THICK, color);
        }
    }
}
