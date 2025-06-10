use macroquad::{
    audio::{
        load_sound_from_bytes, play_sound, play_sound_once, stop_sound, PlaySoundParams, Sound,
    },
    input::{mouse_position, show_mouse},
    math::{Rect, Vec2},
    rand::gen_range,
    text::{draw_text_ex, load_ttf_font_from_bytes, measure_text, Font, TextParams},
    texture::{FilterMode, Texture2D},
    ui::root_ui,
    window::screen_width,
};
use serde_json::Value;

use crate::structs::FONT_COLOR;

pub const END_Y_TEXT: f32 = 17.7;
const CURSOR_SIZE: f32 = 64.0;
const FONT_SIZE: f32 = 24.0;

pub struct Manager {
    pub avaiable: Vec<String>,
    pub font_size: f32,
    pub font: Font,
    cursor_size: Vec2,
    fire_sound1: Sound,
    fire_sound2: Sound,
    fire_sound3: Sound,
    fire_sound4: Sound,
    fire_sound5: Sound,
    reloading_sound1: Sound,
    reloading_sound2: Sound,
    go_voice: Sound,
    hold_voice: Sound,
    mission_completed_voice: Sound,
    ready_voice: Sound,
    cover_me_voice: Sound,
    fire_in_the_hole_voice: Sound,
    get_down_voice: Sound,
    look_out_voice: Sound,
    medic_voice: Sound,
    reloading_voice: Sound,
    sniper_voice: Sound,
    suppressing_fire_voice: Sound,
    watch_my_back_voice: Sound,
    menu_music: Sound,
    closed: Texture2D,
    language: String,
    point: Texture2D,
    local: Value,
}

impl Manager {
    pub async fn new(language: &str, scale: f32) -> Self {
        show_mouse(false);

        let local: Value = serde_json::from_str(include_str!("../assets/local.json"))
            .expect("The localization file is missing");

        let mut font = load_ttf_font_from_bytes(include_bytes!("../assets/font.ttf"))
            .expect("The font file is missing");
        font.set_filter(FilterMode::Linear);

        Self {
            font,
            local: local.clone(),
            avaiable: local
                .as_object()
                .unwrap()
                .iter()
                .map(|lang| String::from(lang.0))
                .collect(),
            language: language.to_string(),
            closed: Texture2D::from_file_with_format(
                include_bytes!("../assets/mouse/closed.png"),
                None,
            ),
            point: Texture2D::from_file_with_format(
                include_bytes!("../assets/mouse/point.png"),
                None,
            ),
            cursor_size: Vec2::splat(CURSOR_SIZE / 4.0 * scale),
            font_size: scale * FONT_SIZE,
            menu_music: load_sound_from_bytes(include_bytes!("../assets/sounds/menu.wav"))
                .await
                .unwrap(),
            fire_sound1: load_sound_from_bytes(include_bytes!("../assets/sounds/gun.wav"))
                .await
                .unwrap(),
            fire_sound2: load_sound_from_bytes(include_bytes!("../assets/sounds/gun1.wav"))
                .await
                .unwrap(),
            fire_sound3: load_sound_from_bytes(include_bytes!("../assets/sounds/shotgun.wav"))
                .await
                .unwrap(),
            fire_sound4: load_sound_from_bytes(include_bytes!("../assets/sounds/gun2.ogg"))
                .await
                .unwrap(),
            fire_sound5: load_sound_from_bytes(include_bytes!("../assets/sounds/gun3.ogg"))
                .await
                .unwrap(),
            reloading_sound1: load_sound_from_bytes(include_bytes!(
                "../assets/sounds/reloading.wav"
            ))
            .await
            .unwrap(),
            reloading_sound2: load_sound_from_bytes(include_bytes!(
                "../assets/sounds/reloading2.wav"
            ))
            .await
            .unwrap(),
            go_voice: load_sound_from_bytes(include_bytes!("../assets/voice/go.ogg"))
                .await
                .unwrap(),
            hold_voice: load_sound_from_bytes(include_bytes!("../assets/voice/hold.ogg"))
                .await
                .unwrap(),
            mission_completed_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/mission_completed.ogg"
            ))
            .await
            .unwrap(),
            ready_voice: load_sound_from_bytes(include_bytes!("../assets/voice/ready.ogg"))
                .await
                .unwrap(),
            cover_me_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_cover_me.ogg"
            ))
            .await
            .unwrap(),
            fire_in_the_hole_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_fire_in_the_hole.ogg"
            ))
            .await
            .unwrap(),
            get_down_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_get_down.ogg"
            ))
            .await
            .unwrap(),
            look_out_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_look_out.ogg"
            ))
            .await
            .unwrap(),
            medic_voice: load_sound_from_bytes(include_bytes!("../assets/voice/war_medic.ogg"))
                .await
                .unwrap(),
            reloading_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_reloading.ogg"
            ))
            .await
            .unwrap(),
            sniper_voice: load_sound_from_bytes(include_bytes!("../assets/voice/war_sniper.ogg"))
                .await
                .unwrap(),
            suppressing_fire_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_suppressing_fire.ogg"
            ))
            .await
            .unwrap(),
            watch_my_back_voice: load_sound_from_bytes(include_bytes!(
                "../assets/voice/war_watch_my_back.ogg"
            ))
            .await
            .unwrap(),
        }
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = String::from(language);
    }

    pub fn start_music(&self) {
        play_sound(
            &self.menu_music,
            PlaySoundParams {
                looped: true,
                volume: 1.0,
            },
        );
    }

    pub fn play_fire(&self, turret: bool) {
        play_sound_once(match gen_range(1, 6) {
            1 => &self.fire_sound1,
            2 => &self.fire_sound2,
            3 => &self.fire_sound3,
            4 => &self.fire_sound4,
            _ => &self.fire_sound5,
        });

        if !turret && gen_range(0, 101) <= 10 {
            play_sound_once(match gen_range(1, 3) {
                1 => &self.reloading_sound1,
                _ => &self.reloading_sound2,
            });
        }

        if !turret && gen_range(0, 100) <= 10 {
            play_sound_once(match gen_range(0, 9) {
                0 => &self.cover_me_voice,
                1 => &self.fire_in_the_hole_voice,
                2 => &self.get_down_voice,
                3 => &self.look_out_voice,
                4 => &self.medic_voice,
                5 => &self.reloading_voice,
                6 => &self.sniper_voice,
                7 => &self.suppressing_fire_voice,
                _ => &self.watch_my_back_voice,
            });
        }
    }

    pub fn play_start_go(&self) {
        play_sound_once(&self.go_voice);
    }

    pub fn play_controlled(&self) {
        play_sound_once(match gen_range(0, 3) {
            0 => &self.ready_voice,
            1 => &self.hold_voice,
            _ => &self.mission_completed_voice,
        });
    }

    pub fn stop_music(&self) {
        stop_sound(&self.menu_music);
    }

    pub fn draw_cursor(&self, closed: bool) {
        let (x, y) = mouse_position();
        root_ui().canvas().image(
            Rect::new(x, y, self.cursor_size.x, self.cursor_size.y),
            if closed { &self.closed } else { &self.point },
        );
    }

    pub fn draw_text(&self, centered: bool, text: &str, y: f32, scale: f32) {
        let font_size = (scale * self.font_size) as u16;
        let font = Some(&self.font);

        draw_text_ex(
            text,
            if centered {
                (screen_width() - measure_text(text, font, font_size, 1.0).width) / 2.0
            } else {
                0.0
            },
            font_size as f32 * (0.75 + y),
            TextParams {
                font,
                font_size,
                color: FONT_COLOR,
                ..Default::default()
            },
        );
    }

    pub fn get_text_by(&self, lang: &str, index: usize) -> &str {
        self.local[lang][index]
            .as_str()
            .unwrap_or_else(|| panic!("The {} language is unavailable ({} code)", lang, index))
    }

    pub fn get_text(&self, index: usize) -> &str {
        self.get_text_by(&self.language, index)
    }
}
