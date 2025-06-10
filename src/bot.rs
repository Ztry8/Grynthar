use macroquad::{math::IVec2, rand::gen_range};

use crate::{
    level::Level,
    manager::Manager,
    squad::Squad,
    structs::{Difficulty, Team, UnitType},
    unit::Unit,
};

pub struct Bot {
    squads: Vec<Squad>,
}

impl Bot {
    pub fn new(level: &mut Level, difficulty: Difficulty) -> Self {
        let coef = difficulty.clone() as usize + 1;
        let goals = level.find_captures();

        let mut base = &IVec2::ZERO;
        while base == &IVec2::ZERO {
            base = &goals[gen_range(0, goals.len() - 1)];
        }

        level.capture(base, &Team::Computer);

        Self {
            squads: (0..coef * goals.len())
                .map(|i| {
                    let mut t = Squad::new(
                        base.with_y(base.y + i as i32),
                        vec![
                            UnitType::Sniper,
                            UnitType::Sniper,
                            UnitType::Infantry,
                            UnitType::Infantry,
                            UnitType::Infantry,
                        ],
                        Team::Computer,
                    );

                    t.set_goal(Some(
                        goals[i / if difficulty == Difficulty::Normal {
                            1
                        } else {
                            2
                        }],
                    ));
                    t
                })
                .collect(),
        }
    }

    fn units(&mut self) -> Vec<&mut Unit> {
        self.squads
            .iter_mut()
            .flat_map(|squad| squad.units())
            .collect()
    }

    pub fn positions(&self) -> Vec<Vec<IVec2>> {
        self.squads
            .iter()
            .map(|squad| squad.positions(true))
            .collect()
    }

    pub fn lose(&self) -> bool {
        self.squads.is_empty()
    }

    pub fn update(
        &mut self,
        manager: &Manager,
        positions: (Vec<IVec2>, Vec<IVec2>, Vec<(IVec2, i16)>),
        level: &mut Level,
        delta: f32,
    ) -> Vec<(IVec2, i16)> {
        let mut units = self.units();
        let mut deleted = Vec::new();

        for pos in &positions.2 {
            if let Some(i) = units.iter().position(|x| x.start_pos == pos.0) {
                units[i].health -= pos.1;
                if units[i].health <= 0 {
                    deleted.push(pos.0);
                }
            }
        }

        let mut delete = Vec::new();
        let mut attacked = Vec::new();
        for (i, squad) in self.squads.iter_mut().enumerate() {
            if let Some(goal) = squad.goal() {
                let path = level.find_path(&positions.0, squad.start_pos(), goal);
                if path.is_empty() {
                    squad.set_goal(None);
                }

                squad.set_path(manager, path);
            }

            attacked.push(
                squad
                    .update(manager, &deleted, &positions.1, level, delta, true)
                    .0,
            );

            if squad.empty() {
                delete.push(i);
            }
        }

        for i in delete {
            self.squads.remove(i);
        }

        attacked.concat()
    }
}
