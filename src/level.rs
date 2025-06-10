use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use macroquad::{
    camera::Camera2D,
    color::Color,
    math::{ivec2, vec2, IVec2, Vec2},
    prelude::info,
    rand::{gen_range, srand},
    shapes::draw_hexagon,
    window::{screen_height, screen_width},
};
use noise::{NoiseFn, Perlin};
use pathfinding::directed::astar::astar;

use crate::{seed, structs::*, TILE_SIZE};

pub const LEVEL_SIZE: i32 = 100;
const HEX_HEIGHT: f32 = (TILE_SIZE + BORDER) * 3.0 / 2.0;
const LEVEL_RANGE: Range<i32> = -LEVEL_SIZE..LEVEL_SIZE;
const HALF_CLEAN_TERRITORY: i32 = CLEAN_TERRITORY / 2;
const HEX_WIDTH: f32 = SQRT_3 * (TILE_SIZE + BORDER);
const BORDERS: f32 = 10.0 * TILE_SIZE;
const CLEAN_TERRITORY: i32 = 20;
const BUILDING_CHANCE: i32 = 25;
const SQRT_3: f32 = 1.7320508;

pub struct Level {
    root: HashMap<IVec2, Tile>,
    visible: HashSet<IVec2>,
    solid: HashSet<IVec2>,
    controls: f32,
}

impl Level {
    pub fn new() -> Self {
        let seed = seed();
        info!("Seed: {}", seed);

        srand(seed);
        let noise = Perlin::new(seed as u32);
        let mut root = HashMap::new();
        let controls = 10;

        for x in LEVEL_RANGE {
            for y in LEVEL_RANGE {
                let height = noise.get([x as f64 * 0.1, y as f64 * 0.1]);
                let pos = ivec2(x, y);

                if height <= -0.5 {
                    root.insert(
                        pos,
                        Tile {
                            r#type: TileType::Water,
                            fill_color: match gen_range(0, 3) {
                                0 => WATER_COLOR,
                                1 => WATER_COLOR_ALT1,
                                _ => WATER_COLOR_ALT2,
                            },
                            border_color: WATER_BORDER_COLOR,
                        },
                    );
                } else if (0.4..=0.6).contains(&height) {
                    root.insert(
                        pos,
                        Tile {
                            r#type: TileType::Mountain,
                            fill_color: HILL_LOW_COLOR,
                            border_color: HILL_LOW_BORDER_COLOR,
                        },
                    );
                } else if (0.6..=0.8).contains(&height) {
                    root.insert(
                        pos,
                        Tile {
                            r#type: TileType::Mountain,
                            fill_color: HILL_MEDIUM_COLOR,
                            border_color: HILL_MEDIUM_BORDER_COLOR,
                        },
                    );
                } else if height > 0.8 {
                    root.insert(
                        pos,
                        Tile {
                            r#type: TileType::Mountain,
                            fill_color: HILL_HIGH_COLOR,
                            border_color: HILL_HIGH_BORDER_COLOR,
                        },
                    );
                } else if gen_range(1, 1001) <= BUILDING_CHANCE && Self::range(&pos) {
                    root.insert(
                        pos,
                        Tile {
                            r#type: TileType::Wall,
                            fill_color: WALL_COLOR,
                            border_color: WALL_BORDER_COLOR,
                        },
                    );
                }
            }
        }

        let mut tcontrols = Vec::new();
        (0..controls).for_each(|i| {
            let mut far: Vec<bool> = vec![true];
            let (mut x, mut y) = (0, 0);

            while far.iter().any(|val| *val) {
                (x, y) = if i != 0 {
                    (
                        gen_range(-LEVEL_SIZE / 2, LEVEL_SIZE / 2),
                        gen_range(-LEVEL_SIZE / 2, LEVEL_SIZE / 2),
                    )
                } else {
                    (0, 0)
                };

                far = tcontrols
                    .iter()
                    .map(|pos| Self::distance(&ivec2(x, y), pos) < 35)
                    .collect();
            }

            tcontrols.push(ivec2(x, y));

            for tx in x - HALF_CLEAN_TERRITORY..x + HALF_CLEAN_TERRITORY {
                for ty in y - HALF_CLEAN_TERRITORY..y + HALF_CLEAN_TERRITORY {
                    root.remove(&ivec2(tx, ty));
                }
            }

            let hex = ivec2(x, y);
            root.insert(
                hex,
                Tile {
                    r#type: TileType::Control(None),
                    fill_color: CAPTURE_UNOCCUPIED_COLOR,
                    border_color: CAPTURE_UNOCCUPIED_BORDER_COLOR,
                },
            );
        });

        Self {
            root: root.clone(),
            solid: HashSet::from_iter(
                root.iter()
                    .filter(|(_, hex)| hex.r#type != TileType::Water)
                    .map(|(pos, _)| *pos),
            ),
            visible: HashSet::new(),
            controls: controls as f32,
        }
    }

    pub fn can_shot(&self, start: &IVec2, end: &IVec2) -> bool {
        let (start, end) = (start.as_vec2(), end.as_vec2());
        let direction = end - start;
        let len = direction.length();
        let mut i = 0.0;

        while i <= len {
            if self
                .solid
                .contains(&(start + direction * (i / len)).round().as_ivec2())
            {
                return false;
            }

            i += 0.01;
        }

        true
    }

    pub fn control_player(&self) -> u8 {
        (self
            .root
            .values()
            .filter(|hex| hex.r#type == TileType::Control(Some(Team::Player)))
            .count() as f32
            / self.controls
            * 100.0) as u8
    }

    pub fn get(&self, pos: &IVec2) -> bool {
        self.root.contains_key(pos)
    }

    pub fn set(&mut self, pos: &IVec2) {
        self.root.insert(
            *pos,
            Tile {
                r#type: TileType::Wall,
                fill_color: WALL_COLOR,
                border_color: WALL_BORDER_COLOR,
            },
        );

        self.solid.insert(*pos);
    }

    pub fn delete(&mut self, pos: &IVec2) {
        self.root.remove(pos);
        self.solid.remove(pos);
    }

    pub fn is_visible(&self, pos: &IVec2) -> bool {
        self.visible.contains(pos)
    }

    pub fn visible(&mut self, pos: IVec2, distance: i32, add: bool) {
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                if (-distance..=distance).contains(&(-dx - dy)) {
                    let tpos = ivec2(dx, dy) + pos;
                    if add {
                        self.visible.insert(tpos);
                    } else {
                        self.visible.remove(&tpos);
                    }
                }
            }
        }
    }

    pub fn capture(&mut self, pos: &IVec2, team: &Team) {
        let hex = self.root.get_mut(pos).unwrap();

        (hex.fill_color, hex.border_color) = match team {
            Team::Player => (PLAYER_COLOR, PLAYER_BORDER_COLOR),
            Team::Computer => (ENEMY_COLOR, ENEMY_BORDER_COLOR),
        };

        hex.r#type = TileType::Control(Some(team.clone()));
        self.visible(*pos, 15, team == &Team::Player);
    }

    pub fn is_capturable(&self, pos: &IVec2, team: &Team) -> bool {
        self.root.contains_key(pos)
            && (self.root[pos].r#type == TileType::Control(None)
                || self.root[pos].r#type == TileType::Control(Some(invert_team(team))))
    }

    pub fn range(pos: &IVec2) -> bool {
        LEVEL_RANGE.contains(&pos.y) && LEVEL_RANGE.contains(&pos.x)
    }

    pub fn neighbours(pos: &IVec2) -> Vec<IVec2> {
        DIRECTIONS.iter().map(|p| *p + *pos).collect()
    }

    pub fn convert(x: f32, y: f32) -> (f32, f32) {
        (HEX_WIDTH * (x + y / 2.0), HEX_HEIGHT * y)
    }

    pub fn wall(&self, pos: &IVec2) -> bool {
        if self.root.contains_key(pos) {
            self.root[pos].r#type == TileType::Wall
        } else {
            false
        }
    }

    pub fn hex(pos: Vec2) -> IVec2 {
        (vec2((SQRT_3 * pos.x - pos.y) / 3.0, pos.y * 2.0 / 3.0) / (TILE_SIZE + BORDER))
            .round()
            .as_ivec2()
    }

    pub fn distance(a: &IVec2, b: &IVec2) -> u32 {
        a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
    }

    pub fn find_captures(&self) -> Vec<IVec2> {
        self.root
            .iter()
            .filter(|hex| {
                hex.1.r#type == TileType::Control(None)
                    || hex.1.r#type == TileType::Control(Some(Team::Computer))
                    || hex.1.r#type == TileType::Control(Some(Team::Player))
            })
            .map(|hex| *hex.0)
            .collect::<Vec<IVec2>>()
    }

    pub fn draw_hex(pos: &IVec2, border: Color, fill: Color) {
        let (x, y) = Self::convert(pos.x as f32, pos.y as f32);
        draw_hexagon(x, y, TILE_SIZE, BORDER, true, border, fill);
    }

    pub fn find_path(&self, units: &[IVec2], start: IVec2, mut goal: IVec2) -> Vec<IVec2> {
        if self.root.contains_key(&goal) {
            let result = Self::neighbours(&goal)
                .into_iter()
                .filter(|hex| !self.root.contains_key(hex))
                .collect::<Vec<IVec2>>();

            if !result.is_empty() {
                goal = result[0];
            } else {
                return Vec::new();
            }
        }

        astar(
            &start,
            |pos| {
                Self::neighbours(pos)
                    .into_iter()
                    .filter(|hex| {
                        Self::range(hex) && !self.root.contains_key(hex) && !units.contains(hex)
                    })
                    .map(|hex| (hex, 0))
                    .collect::<Vec<(IVec2, u32)>>()
            },
            |pos| Self::distance(pos, &goal) / 2,
            |pos| *pos == goal,
        )
        .unwrap_or_default()
        .0
    }

    pub fn update(&mut self, camera: &Camera2D) {
        for pos in &self.visible {
            let (x, y) = Self::convert(pos.x as f32, pos.y as f32);
            let point = camera.world_to_screen(vec2(x, y));

            if (-BORDERS..screen_width() + BORDERS).contains(&point.x)
                && (-BORDERS..screen_height() + BORDERS).contains(&point.y)
            {
                if self.root.contains_key(pos) {
                    let hex = &self.root[pos];
                    Self::draw_hex(pos, hex.border_color, hex.fill_color);
                } else {
                    Self::draw_hex(
                        pos,
                        GRASS_COLOR,
                        if pos.x.abs() == LEVEL_SIZE || pos.y.abs() == LEVEL_SIZE {
                            HILL_HIGH_BORDER_COLOR
                        } else {
                            GRASS_COLOR
                        },
                    );
                }
            }
        }
    }
}
