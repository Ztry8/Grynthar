use crate::{
    level::Level,
    manager::Manager,
    structs::{Team, UnitType, BORDER, WALL_BORDER_COLOR},
    unit::{Unit, UNIT_SIZE},
};
use macroquad::{
    math::{ivec2, IVec2},
    shapes::draw_arc,
};

const VISIBLE_DISTANCE: i32 = 2;
const CAPTURE_TIME: f32 = 5.0;

#[derive(PartialEq, Clone)]
pub enum Action {
    Destroy,
    Turret,
    Wall,
    Go,
}

#[derive(PartialEq, Clone)]
pub struct Squad {
    action: Option<Action>,
    soldiers: Vec<Unit>,
    goal: Option<IVec2>,
    orig: Option<IVec2>,
    timer_capture: f32,
    path: Vec<IVec2>,
    timer_build: f32,
    timer_heal: f32,
    sound: bool,
    team: Team,
}

impl Squad {
    pub fn new(start: IVec2, body: Vec<UnitType>, team: Team) -> Self {
        Self {
            soldiers: body
                .iter()
                .enumerate()
                .map(|(dt, r#type)| (ivec2(start.x + dt as i32, start.y), r#type))
                .rev()
                .map(|(pos, r#type)| {
                    Unit::new(
                        r#type,
                        pos,
                        0.1 / if r#type == &UnitType::Scout { 1.2 } else { 1.0 },
                    )
                })
                .collect(),
            timer_capture: 0.0,
            path: Vec::new(),
            timer_build: 0.0,
            timer_heal: 0.0,
            action: if body.contains(&UnitType::Engineer) {
                None
            } else {
                Some(Action::Go)
            },
            sound: true,
            goal: None,
            orig: None,
            team,
        }
    }

    pub fn engineer(&self) -> bool {
        self.soldiers
            .iter()
            .any(|unit| unit.r#type == UnitType::Engineer)
    }

    pub fn units(&mut self) -> Vec<&mut Unit> {
        self.soldiers.iter_mut().collect()
    }

    pub fn set_goal(&mut self, goal: Option<IVec2>) {
        self.timer_build = 0.0;
        self.sound = true;
        self.goal = goal;
    }

    pub fn set_action(&mut self, action: Action) {
        self.timer_build = 0.0;
        self.action = Some(action);
    }

    pub fn action(&self) -> Action {
        if let Some(action) = self.action.clone() {
            action
        } else {
            Action::Go
        }
    }

    pub fn busy(&self) -> bool {
        self.action.is_some()
    }

    pub fn goal(&self) -> Option<IVec2> {
        self.goal
    }

    pub fn rev(&mut self) {
        self.soldiers = self
            .soldiers
            .iter()
            .rev()
            .map(|unit| unit.clone())
            .collect();
    }

    pub fn set_path(&mut self, manager: &Manager, path: Vec<IVec2>) {
        self.path = path;
        if !self.path.is_empty() {
            self.path.remove(0);
            if self.sound && self.team == Team::Player {
                manager.play_start_go();
                self.sound = false;
            }

            if !self.path.is_empty() {
                self.soldiers[0].end_pos = self.path[0];
            }
        } else {
            self.goal = None;
        }
    }

    pub fn empty(&self) -> bool {
        self.soldiers.is_empty()
    }

    pub fn start_pos(&self) -> IVec2 {
        self.soldiers[0].start_pos
    }

    pub fn positions(&self, turret: bool) -> Vec<IVec2> {
        if turret {
            self.soldiers.iter().map(|unit| unit.start_pos).collect()
        } else {
            self.soldiers
                .iter()
                .filter(|unit| unit.r#type != UnitType::Turret)
                .map(|unit| unit.start_pos)
                .collect()
        }
    }

    pub fn update(
        &mut self,
        manager: &Manager,
        deleted: &[IVec2],
        positions: &[IVec2],
        level: &mut Level,
        delta: f32,
        active: bool,
    ) -> (Vec<(IVec2, i16)>, i32, i32) {
        let mut start_pos = if !self.path.is_empty() {
            Some(self.path[0])
        } else {
            None
        };

        for pos in deleted {
            if let Some(i) = self.soldiers.iter().position(|x| x.start_pos == *pos) {
                self.soldiers.remove(i);
            }
        }

        let (mut created, mut turrets) = (0, 0);
        let (wall, turret, destroy) = (
            self.action == Some(Action::Wall),
            self.action == Some(Action::Turret),
            self.action == Some(Action::Destroy),
        );

        if wall
            || turret
            || (destroy
                && if let Some(goal) = self.goal {
                    level.wall(&goal)
                } else {
                    true
                })
        {
            if self.timer_build > 0.0 {
                self.timer_build += delta;

                if let Some(orig) = self.orig {
                    let (x, y) = Level::convert(orig.x as f32, orig.y as f32);

                    draw_arc(
                        x,
                        y,
                        20,
                        UNIT_SIZE,
                        0.0,
                        BORDER,
                        self.timer_build / 1.0 * 360.0,
                        WALL_BORDER_COLOR,
                    );

                    if self.timer_build >= 1.0 {
                        if wall {
                            created += 1;
                            level.set(&orig);
                        } else if destroy {
                            created -= 1;
                            level.delete(&orig);
                        } else if turret {
                            turrets += 1;
                            self.soldiers.push(Unit::new(&UnitType::Turret, orig, 0.0));
                        }

                        self.timer_build = 0.0;
                        self.orig = None;
                    }
                }
            }

            if let Some(goal) = self.goal {
                if Level::neighbours(&self.start_pos()).contains(&goal) {
                    self.path.clear();
                    self.orig = self.goal;
                    self.goal = None;
                    self.timer_build += delta;
                }
            }
        }

        self.timer_heal += delta;
        let heal = if self.timer_heal >= 1.0 {
            self.timer_heal = 0.0;
            self.soldiers
                .iter()
                .any(|unit| unit.r#type == UnitType::Medic)
        } else {
            false
        };

        let mut attacked = Vec::new();
        let soldiers_len = self.soldiers.len() as i32;
        self.soldiers.iter_mut().for_each(|unit| {
            if self.team == Team::Player {
                level.visible(
                    unit.start_pos,
                    if unit.r#type == UnitType::Scout {
                        soldiers_len
                    } else {
                        1
                    } * VISIBLE_DISTANCE,
                    true,
                );
            }

            if heal && unit.r#type != UnitType::Turret {
                unit.heal();
            }

            if !positions.is_empty() {
                let mut positions = positions
                    .iter()
                    .map(|a| (Level::distance(a, &unit.start_pos), *a))
                    .collect::<Vec<(u32, IVec2)>>();
                positions.sort_by(|a, b| a.0.cmp(&b.0));

                let pos = positions[0];
                let sniper = unit.r#type == UnitType::Sniper;
                let visible = level.is_visible(&pos.1) || sniper;
                if unit.fire(if visible { Some(pos.1) } else { None }, delta)
                    && visible
                    && pos.0 < 15 * if sniper { 2 } else { 1 }
                    && level.can_shot(&unit.start_pos, &pos.1)
                {
                    unit.zero_timer();
                    attacked.push((
                        pos.1,
                        match unit.r#type {
                            UnitType::Infantry => 15,
                            UnitType::Engineer => 10,
                            UnitType::Scout => 12,
                            UnitType::Medic | UnitType::Turret => 8,
                            UnitType::Sniper => 50,
                        },
                    ));
                    manager.play_fire(unit.r#type == UnitType::Turret);
                }
            }

            if unit.r#type != UnitType::Turret {
                if let Some(pos) = start_pos {
                    if !self.path.is_empty() {
                        if unit.update(delta) {
                            unit.end_pos = pos;
                        }
                    } else {
                        self.goal = None;
                    }

                    start_pos = Some(unit.start_pos);
                }
            }

            Level::neighbours(&unit.start_pos).iter().for_each(|hex| {
                if level.is_capturable(hex, &self.team) {
                    self.timer_capture += delta;
                    if self.timer_capture >= CAPTURE_TIME {
                        level.capture(hex, &self.team);
                        if unit.r#type != UnitType::Turret && self.team == Team::Player {
                            manager.play_controlled();
                        }
                        self.timer_capture = 0.0;
                    }
                }
            });

            if level.is_visible(&unit.start_pos) {
                unit.render(&self.team, active);
            }
        });

        (attacked, created, turrets)
    }
}
