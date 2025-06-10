use ini::Ini;

const INI_GAME_NAME: &str = "game.ini";
const INI_GRAPHICS_NAME: &str = "graphics.ini";
pub const INI_GAME_ERROR: &str = "Check the \"game.ini\" file";
pub const INI_GRAPHICS_ERROR: &str = "Check the \"graphics.ini\" file";

pub const SCALE_FIELD_NAME: &str = "Scale";
pub const HIGHDPI_FIELD_NAME: &str = "HighDPI";
pub const FULLSCREEN_FIELD_NAME: &str = "FullScreen";
pub const ANTIALIASING_FIELD_NAME: &str = "Antialiasing";

pub const SENSITIVITY_FIELD_NAME: &str = "Sensitivity";

pub fn gen_graphics() -> Ini {
    Ini::load_from_file(INI_GRAPHICS_NAME).unwrap_or_else(|_| {
        let mut file = Ini::new();
        file.with_general_section()
            .set(SCALE_FIELD_NAME, "1")
            .set(HIGHDPI_FIELD_NAME, "false")
            .set(FULLSCREEN_FIELD_NAME, "false")
            .set(ANTIALIASING_FIELD_NAME, "2");
        #[cfg(not(target_arch = "wasm32"))]
        file.write_to_file(INI_GRAPHICS_NAME).unwrap();
        file
    })
}

pub fn gen_game() -> Ini {
    Ini::load_from_file(INI_GAME_NAME).unwrap_or_else(|_| {
        let mut file = Ini::new();
        file.with_general_section()
            .set(SENSITIVITY_FIELD_NAME, "100");
        #[cfg(not(target_arch = "wasm32"))]
        file.write_to_file(INI_GAME_NAME).unwrap();
        file
    })
}
