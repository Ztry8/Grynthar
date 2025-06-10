use macroquad::{
    camera::set_default_camera,
    math::{ivec2, IVec2},
};

use crate::{
    level::Level,
    manager::Manager,
    menu::Menu,
    squad::{Action, Squad},
    structs::{Team, UnitType},
    unit::Unit,
};

pub struct Player {
    pub fine: u32,
    walls: i32,
    turrets: i32,
    squads: Vec<Squad>,
    current: Option<usize>,
}

impl Player {
    pub fn new(units: Vec<Vec<UnitType>>) -> Self {
        Self {
            squads: units
                .into_iter()
                .enumerate()
                .map(|(i, mut types)| {
                    types.reverse();
                    Squad::new(
                        ivec2(0, if i == 0 { 1 } else { -(i as i32) }),
                        types,
                        Team::Player,
                    )
                })
                .collect(),
            turrets: 0,
            walls: 0,
            current: None,
            fine: 0,
        }
    }

    fn units(&mut self) -> Vec<&mut Unit> {
        self.squads
            .iter_mut()
            .flat_map(|squad| squad.units())
            .collect()
    }

    pub fn lose(&mut self) -> bool {
        if self.squads.len() == 1 && self.squads[0].engineer() {
            let mut lose = true;
            for unit in self.squads[0].units() {
                if unit.r#type == UnitType::Engineer {
                    lose = false;
                    break;
                }
            }

            lose
        } else {
            self.squads.is_empty()
        }
    }

    pub fn update(
        &mut self,
        menu: &Menu,
        manager: &Manager,
        attacked: &Vec<(IVec2, i16)>,
        mut positions: Vec<Vec<IVec2>>,
        level: &mut Level,
        goal: IVec2,
        mut clicked: bool,
        delta: f32,
    ) -> (Vec<IVec2>, Vec<IVec2>, Vec<(IVec2, i16)>) {
        let mut units = self.units();
        let mut deleted = Vec::new();
        let mut turrets = 0;
        let mut fine = 0;

        for pos in attacked {
            if let Some(i) = units.iter().position(|x| x.start_pos == pos.0) {
                units[i].health -= pos.1;
                if units[i].health <= 0 {
                    deleted.push(pos.0);

                    fine = match units[i].r#type {
                        UnitType::Engineer => 5,
                        UnitType::Turret => {
                            turrets -= 1;
                            0
                        }
                        _ => 0,
                    };
                }
            }
        }

        self.fine += fine;
        self.turrets += turrets;
        let attack_positions = &positions.concat();
        let mut player_units = Vec::new();

        for (i, squad) in self.squads.iter().enumerate() {
            let tpositions = squad.positions(true);
            if clicked && squad.positions(false).contains(&goal) {
                let real = Some(i);
                clicked = false;

                self.current = if self.current == real { None } else { real }
            }

            player_units.push(tpositions.clone());
            positions.push(tpositions);
        }

        if let Some(i) = self.current {
            if i < self.squads.len() && self.squads[i].engineer() {
                let squad = &mut self.squads[i];

                if menu.button(0.0, 12.0, manager.get_text(22)) {
                    squad.set_action(Action::Go);
                    clicked = false;
                }

                if self.walls < 10 && menu.button(0.0, 13.0, manager.get_text(23)) {
                    squad.set_action(Action::Wall);
                    clicked = false;
                }

                if self.turrets < 5 && menu.button(0.0, 14.0, manager.get_text(24)) {
                    squad.set_action(Action::Turret);
                    clicked = false;
                }

                if menu.button(0.0, 15.0, manager.get_text(25)) {
                    squad.set_action(Action::Destroy);
                    clicked = false;
                }

                let action = squad.action();
                if action == Action::Turret && self.turrets >= 5 {
                    squad.set_action(Action::Go);
                }

                if action == Action::Wall && self.walls >= 10 {
                    squad.set_action(Action::Go);
                }
            }
        }

        let mut delete = Vec::new();
        let mut attacked = Vec::new();
        let positions = positions.concat();
        for (i, squad) in &mut self.squads.iter_mut().enumerate() {
            if let Some(current) = self.current {
                if clicked && i == current && squad.busy() {
                    let action = squad.action();
                    if action == Action::Go
                        || (action == Action::Destroy && level.wall(&goal))
                        || (action != Action::Destroy && !level.get(&goal))
                    {
                        squad.set_goal(Some(goal));
                    }
                }
            }

            if let Some(goal) = squad.goal() {
                let mut path = level.find_path(&positions, squad.start_pos(), goal);
                if path.is_empty() {
                    squad.rev();
                    path = level.find_path(&positions, squad.start_pos(), goal);
                }

                squad.set_path(manager, path);
            }

            let squad_result = squad.update(
                manager,
                &deleted,
                attack_positions,
                level,
                delta,
                if let Some(current) = self.current {
                    current == i
                } else {
                    false
                },
            );

            self.walls += squad_result.1;
            self.turrets += squad_result.2;
            attacked.push(squad_result.0);

            if squad.empty() {
                delete.push(i);
            }
        }

        for i in delete {
            if Some(i) == self.current {
                self.current = None;
            }
            self.squads.remove(i);
        }

        (positions, player_units.concat(), attacked.concat())
    }

    pub fn draw_ui(&self, manager: &Manager, closed: bool, controls: u8) {
        set_default_camera();
        manager.draw_text(
            false,
            &format!("{}% {}", controls, manager.get_text(0)),
            0.1,
            1.0,
        );
        manager.draw_cursor(closed);

        #[cfg(debug_assertions)]
        {
            use crate::manager::END_Y_TEXT;
            use macroquad::time::get_fps;
            manager.draw_text(false, &format!("FPS:{}", get_fps()), END_Y_TEXT, 1.0);
        }
    }
}
