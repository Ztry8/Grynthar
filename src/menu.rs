use macroquad::{
    math::vec2,
    rand::gen_range,
    text::measure_text,
    ui::{root_ui, Skin},
    window::screen_width,
};

use crate::{
    game::GameResult,
    manager::{Manager, END_Y_TEXT},
    structs::{Difficulty, UnitType, FOG_COLOR, FONT_COLOR, HOVERED_COLOR},
};

enum MenuState {
    Construct(Difficulty),
    Difficulty,
    End(usize),
    Language,
}

pub struct Menu {
    pub game_timer: f64,
    army: Vec<Vec<UnitType>>,
    lang: Option<String>,
    custom_squad: bool,
    state: MenuState,
    author: String,
    game_time: u64,
    font_size: f32,
    title: String,
}

impl Menu {
    pub fn new(manager: &Manager) -> Self {
        manager.start_music();
        let font_size = manager.font_size * 1.0;
        let style = root_ui()
            .style_builder()
            .color(FOG_COLOR)
            .color_clicked(FOG_COLOR)
            .color_hovered(FOG_COLOR)
            .color_inactive(FOG_COLOR)
            .color_selected(FOG_COLOR)
            .color_selected_hovered(FOG_COLOR)
            .text_color(FONT_COLOR)
            .text_color_hovered(HOVERED_COLOR)
            .text_color_clicked(FONT_COLOR)
            .font_size(font_size as u16)
            .font(include_bytes!("../assets/font.ttf"))
            .unwrap()
            .build();

        let skin = Skin {
            button_style: style,
            ..root_ui().default_skin()
        };

        root_ui().push_skin(&skin);

        Self {
            font_size,
            lang: None,
            game_time: 0,
            army: Vec::new(),
            custom_squad: false,
            state: MenuState::Language,
            game_timer: 0.0,
            author: format!("By {}", env!("CARGO_PKG_AUTHORS")),
            title: format!("Grynthar Beta {}", env!("CARGO_PKG_VERSION")),
        }
    }

    fn button_centered(&self, manager: &Manager, y: f32, text: &str) -> bool {
        root_ui().button(
            vec2(
                (screen_width()
                    - measure_text(text, Some(&manager.font), self.font_size as u16, 1.0).width)
                    / 2.0,
                self.font_size * y,
            ),
            text,
        )
    }

    pub fn get_units(&mut self) -> Vec<Vec<UnitType>> {
        let result = self.army.clone();
        self.army.clear();
        result
    }

    pub fn button(&self, x: f32, y: f32, text: &str) -> bool {
        root_ui().button(vec2(screen_width() * x, self.font_size * (0.75 + y)), text)
    }

    pub fn update(&mut self, manager: &mut Manager, result: &GameResult) -> Option<Difficulty> {
        manager.draw_text(false, &self.title, 0.1, 1.5);
        manager.draw_text(false, &self.author, END_Y_TEXT, 1.0);

        let result = match &self.state {
            MenuState::Difficulty => {
                if let Some(lang) = &self.lang {
                    manager.set_language(lang);
                    self.lang = None;
                }

                if self.button_centered(manager, 6.0, manager.get_text(1)) {
                    self.game_timer = 0.0;
                    self.state = MenuState::Construct(Difficulty::Normal);
                }

                if self.button_centered(manager, 7.5, manager.get_text(2)) {
                    self.game_timer = 0.0;
                    self.state = MenuState::Construct(Difficulty::Hard);
                }

                None
            }
            MenuState::Construct(difficulty) => {
                manager.draw_text(true, manager.get_text(28), 8.5, 1.0);
                let army_len = self.army.len();
                let (empty, len) = if self.custom_squad {
                    if army_len > 0 {
                        (
                            self.army[army_len - 1].is_empty(),
                            self.army[army_len - 1].len(),
                        )
                    } else {
                        (true, 0)
                    }
                } else {
                    (self.army.is_empty(), army_len)
                };

                if !empty {
                    if self.custom_squad {
                        for (i, unit) in self.army[army_len - 1].iter().enumerate() {
                            manager.draw_text(
                                true,
                                &format!(
                                    "{}) {}",
                                    i + 1,
                                    match unit {
                                        UnitType::Sniper => manager.get_text(35),
                                        UnitType::Medic => manager.get_text(34),
                                        _ => manager.get_text(33),
                                    }
                                ),
                                9.5 + i as f32,
                                1.0,
                            );
                        }
                    } else {
                        let mut text;
                        for (i, squad) in self.army.iter().enumerate() {
                            manager.draw_text(
                                true,
                                &format!(
                                    "{}) {}",
                                    i + 1,
                                    match squad[0] {
                                        UnitType::Engineer => manager.get_text(30),
                                        UnitType::Scout => manager.get_text(32),
                                        _ => {
                                            text = format!("{}:", manager.get_text(31));
                                            for (i, r#type) in [
                                                UnitType::Infantry,
                                                UnitType::Medic,
                                                UnitType::Sniper,
                                            ]
                                            .iter()
                                            .enumerate()
                                            {
                                                let count = squad
                                                    .iter()
                                                    .filter(|unit| unit == &r#type)
                                                    .count();
                                                if count > 0 {
                                                    text = format!(
                                                        "{} {} ({}x),",
                                                        text,
                                                        manager.get_text(33 + i),
                                                        count
                                                    );
                                                }
                                            }

                                            &text
                                        }
                                    }
                                ),
                                9.5 + i as f32,
                                1.0,
                            );
                        }
                    }
                } else {
                    manager.draw_text(true, manager.get_text(36), 9.5, 1.0);
                }

                if len
                    >= if difficulty == &Difficulty::Normal {
                        7
                    } else {
                        5
                    }
                {
                    if self.button_centered(
                        manager,
                        3.5,
                        manager.get_text(if self.custom_squad { 39 } else { 37 }),
                    ) {
                        if self.custom_squad {
                            self.custom_squad = false;
                        } else {
                            let diff = difficulty.clone();
                            self.state = MenuState::End(gen_range(0, 8));
                            return Some(diff);
                        }
                    }

                    if self.button_centered(manager, 4.8, manager.get_text(38)) {
                        if self.custom_squad {
                            self.army[army_len - 1].clear();
                        } else {
                            self.army.clear();
                        }
                    }
                } else {
                    manager.draw_text(
                        true,
                        manager.get_text(if self.custom_squad { 41 } else { 29 }),
                        2.0,
                        1.0,
                    );

                    if self.button_centered(
                        manager,
                        3.0,
                        manager.get_text(if self.custom_squad { 33 } else { 30 }),
                    ) {
                        if self.custom_squad {
                            self.army[army_len - 1].push(UnitType::Infantry);
                        } else {
                            self.army.push(
                                (0..if difficulty == &Difficulty::Normal {
                                    4
                                } else {
                                    3
                                })
                                    .map(|_| UnitType::Engineer)
                                    .collect(),
                            );
                        }
                    }

                    if self.button_centered(
                        manager,
                        4.0,
                        manager.get_text(if self.custom_squad { 34 } else { 31 }),
                    ) {
                        if self.custom_squad {
                            self.army[army_len - 1].push(UnitType::Medic);
                        } else {
                            self.custom_squad = true;
                            self.army.push(Vec::new());
                        }
                    }

                    if self.button_centered(
                        manager,
                        5.2,
                        manager.get_text(if self.custom_squad { 35 } else { 32 }),
                    ) {
                        if self.custom_squad {
                            self.army[army_len - 1].push(UnitType::Sniper);
                        } else {
                            self.army.push(
                                (0..if difficulty == &Difficulty::Normal {
                                    5
                                } else {
                                    4
                                })
                                    .map(|_| UnitType::Scout)
                                    .collect(),
                            );
                        }
                    }

                    if self.button_centered(manager, 6.7, manager.get_text(40)) {
                        if self.custom_squad {
                            if !empty {
                                self.army[army_len - 1].pop();
                            }
                        } else {
                            self.army.pop();
                        }
                    }
                }

                None
            }
            MenuState::End(advice) => {
                if self.game_time == 0 {
                    self.game_time = self.game_timer as u64;
                }

                manager.draw_text(
                    true,
                    &format!(
                        "{}: {} {}",
                        &manager.get_text(20),
                        self.game_time / 60,
                        &manager.get_text(21)
                    ),
                    5.0,
                    1.0,
                );

                match *result {
                    GameResult::Lose(fine) => {
                        manager.draw_text(true, manager.get_text(19), 1.0, 2.0);
                        manager.draw_text(
                            true,
                            &format!("{}: {}", manager.get_text(26), fine),
                            6.5,
                            1.0,
                        );

                        manager.draw_text(
                            true,
                            &format!("{}: {}", manager.get_text(7), manager.get_text(8 + advice)),
                            12.0,
                            1.0,
                        )
                    }
                    GameResult::Win(fine) => {
                        manager.draw_text(true, manager.get_text(18), 1.0, 2.0);
                        manager.draw_text(
                            true,
                            &format!("{}: {}", manager.get_text(26), fine),
                            6.5,
                            1.0,
                        );
                    }
                }

                if self.button_centered(manager, 13.0, manager.get_text(6)) {
                    self.game_time = 0;
                    self.state = MenuState::Difficulty;
                }

                None
            }
            MenuState::Language => {
                for (i, lang) in manager.avaiable.iter().enumerate() {
                    if self.button(
                        if i < 9 {
                            0.05
                        } else if i < 18 {
                            0.3
                        } else {
                            0.55
                        },
                        1.5 + (i as f32
                            - if i >= 18 {
                                18.0
                            } else if i >= 9 {
                                9.0
                            } else {
                                0.0
                            })
                            * 1.7,
                        lang,
                    ) {
                        self.state = MenuState::Difficulty;
                        self.lang = Some(lang.to_owned());
                    }
                }
                None
            }
        };

        manager.draw_cursor(false);
        result
    }
}
