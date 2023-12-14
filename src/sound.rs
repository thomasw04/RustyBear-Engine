#[cfg(not(target_arch = "wasm32"))]
use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::{
        static_sound::{StaticSoundData, StaticSoundSettings},
        streaming::{StreamingSoundData, StreamingSoundSettings},
    },
};

use crate::environment::config::ThemeConfiguration;

#[cfg(not(target_arch = "wasm32"))]
pub struct AudioEngine {
    manager: kira::manager::AudioManager,
    background_music: String,
    //click_sound: StaticSoundData,
}

#[cfg(target_arch = "wasm32")]
pub struct AudioEngine {}

#[cfg(not(target_arch = "wasm32"))]
impl AudioEngine {
    pub fn new(theme_conf: &ThemeConfiguration) -> Self {
        AudioEngine {
            manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap(),
            background_music: theme_conf.background_music.clone(),
        }
    }

    pub fn play_click(&mut self) {
        //let mut sound = self.manager.play(self.click_sound.clone()).unwrap();
        //let _ = sound.set_volume(0.1, kira::tween::Tween::default());
    }

    pub fn play_background(&mut self) {
        let sound_data_res = StreamingSoundData::from_file(
            format!("themes/{}", self.background_music),
            StreamingSoundSettings::new().loop_region(0.0..),
        );

        if let Ok(sound_data) = sound_data_res {
            let mut sound = self.manager.play(sound_data).unwrap();
            let _ = sound.set_volume(kira::Volume::Decibels(-20.0), kira::tween::Tween::default());
        } else {
            log::error!(
                "Could not load background music {}. Silence.",
                self.background_music
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioEngine {
    pub fn new(theme_conf: &ThemeConfiguration) -> Self {
        AudioEngine {}
    }

    pub fn play_click(&mut self) {}

    pub fn play_background(&mut self) {}
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(&ThemeConfiguration::default())
    }
}
