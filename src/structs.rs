use macroquad::{
    color::Color,
    color_u8,
    math::{ivec2, IVec2},
};

pub const BORDER: f32 = 0.2;
pub const FOG_COLOR: Color = color_u8!(35, 45, 55, 255);
pub const GRASS_COLOR: Color = color_u8!(102, 153, 51, 255);

pub const FONT_COLOR: Color = color_u8!(226, 233, 233, 255);
pub const HOVERED_COLOR: Color = color_u8!(180, 190, 190, 255);

pub const WALL_COLOR: Color = color_u8!(80, 80, 80, 255);
pub const WALL_BORDER_COLOR: Color = color_u8!(60, 60, 60, 255);

pub const HILL_LOW_COLOR: Color = color_u8!(80, 130, 60, 255);
pub const HILL_LOW_BORDER_COLOR: Color = color_u8!(60, 100, 40, 255);

pub const HILL_MEDIUM_COLOR: Color = color_u8!(70, 110, 50, 255);
pub const HILL_MEDIUM_BORDER_COLOR: Color = color_u8!(50, 80, 35, 255);

pub const HILL_HIGH_COLOR: Color = color_u8!(60, 90, 40, 255);
pub const HILL_HIGH_BORDER_COLOR: Color = color_u8!(40, 65, 30, 255);

pub const WATER_COLOR: Color = color_u8!(27, 150, 186, 255);
pub const WATER_COLOR_ALT1: Color = color_u8!(37, 160, 196, 255);
pub const WATER_COLOR_ALT2: Color = color_u8!(17, 140, 176, 255);
pub const WATER_BORDER_COLOR: Color = color_u8!(18, 120, 155, 255);

pub const CAPTURE_UNOCCUPIED_COLOR: Color = color_u8!(210, 210, 210, 255);
pub const CAPTURE_UNOCCUPIED_BORDER_COLOR: Color = color_u8!(150, 150, 150, 255);

pub const PLAYER_COLOR: Color = color_u8!(100, 180, 255, 255);
pub const PLAYER_BORDER_COLOR: Color = color_u8!(33, 75, 128, 255);

pub const ENEMY_COLOR: Color = color_u8!(203, 47, 44, 255);
pub const ENEMY_BORDER_COLOR: Color = color_u8!(128, 30, 28, 255);

pub const SNIPER_PLAYER_COLOR: Color = color_u8!(120, 140, 255, 255);
pub const SNIPER_PLAYER_BORDER_COLOR: Color = color_u8!(80, 60, 150, 255);

pub const SNIPER_ENEMY_COLOR: Color = color_u8!(220, 100, 60, 255);
pub const SNIPER_ENEMY_BORDER_COLOR: Color = color_u8!(150, 70, 40, 255);

#[derive(Clone)]
pub struct Tile {
    pub r#type: TileType,
    pub fill_color: Color,
    pub border_color: Color,
}

#[derive(PartialEq, Clone)]
pub enum TileType {
    Control(Option<Team>),
    Mountain,
    Water,
    Wall,
}

#[derive(PartialEq, Clone)]
pub enum Team {
    Player,
    Computer,
}

#[derive(PartialEq, Clone)]
pub enum UnitType {
    Engineer,
    Infantry,
    Turret,
    Scout,
    Medic,
    Sniper,
}

#[derive(PartialEq, Clone)]
pub enum Difficulty {
    Normal,
    Hard,
}

pub fn invert_team(team: &Team) -> Team {
    if team == &Team::Player {
        Team::Computer
    } else {
        Team::Player
    }
}

pub const DIRECTIONS: [IVec2; 6] = [
    ivec2(1, 0),
    ivec2(1, -1),
    ivec2(0, -1),
    ivec2(-1, 0),
    ivec2(-1, 1),
    ivec2(0, 1),
];
