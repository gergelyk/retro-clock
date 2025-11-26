use retro_clock_core::settings_models::SystemSettings;

pub const AMBIENT_LIGHT_MAX: u32 = 950;

#[derive(Debug)]
pub enum IlluminationUpdate {
    AmbientLight(u32),
    Settings(IlluminationSettings),
    Reset,
}

#[derive(PartialEq, Debug)]
pub enum BrightnessSettings {
    Auto,
    Manual { brightness: u8 },
}

impl BrightnessSettings {
    pub fn new(brightness: u8, brightness_auto: bool) -> Self {
        if brightness_auto {
            BrightnessSettings::Auto
        } else {
            BrightnessSettings::Manual { brightness }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct IlluminationSettings {
    pub background: BrightnessSettings,
    pub indicators: BrightnessSettings,
}

impl IlluminationSettings {
    pub fn from_system_settings(system_settings: &SystemSettings) -> Self {
        IlluminationSettings {
            background: BrightnessSettings::new(
                system_settings.bg_brightness,
                system_settings.bg_brightness_auto,
            ),
            indicators: BrightnessSettings::new(
                system_settings.led_brightness,
                system_settings.led_brightness_auto,
            ),
        }
    }
}
