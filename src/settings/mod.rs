use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};
use video::{
    SyltMsaaSetting, SyltPresentModeSetting, SyltUiScaleSetting,
    SyltWindowModeSetting,
};

use crate::{i18n::SyltLocale, signals::SyltSignal};

pub mod language;
pub mod video;
pub mod volume;

pub struct SyltSettingsPlugin;

const SETTINGS_FILE: &str = "settings.yaml";

impl Plugin for SyltSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SettingsEvent>();
        app.add_plugins((
            volume::SyltVolumePlugin,
            language::SyltLanguagePlugin,
            video::SyltVideoPlugin,
        ));
        app.init_resource::<SyltSettings>();
        app.add_systems(Startup, trigger_load_settings);
        app.add_systems(
            Update,
            (
                trigger_save_settings,
                handle_settings_loaded,
                handle_settings_saved,
            ),
        );
    }
}

#[derive(Event)]
pub enum SettingsEvent {
    Save,
    Load,
}

#[derive(Debug, Resource, serde::Deserialize, serde::Serialize)]
pub struct SyltSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub effects_volume: f32,
    pub dialog_volume: f32,
    pub ui_volume: f32,
    pub locale: SyltLocale,
    pub msaa_setting: SyltMsaaSetting,
    pub window_mode: SyltWindowModeSetting,
    pub vsync: SyltPresentModeSetting,
    pub ui_scale: SyltUiScaleSetting,
}

impl Default for SyltSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.25,
            dialog_volume: 1.0,
            music_volume: 0.8,
            effects_volume: 0.9,
            ui_volume: 0.7,
            locale: SyltLocale::English,
            msaa_setting: SyltMsaaSetting(Msaa::Off),
            window_mode: SyltWindowModeSetting(WindowMode::Windowed),
            vsync: SyltPresentModeSetting(PresentMode::AutoNoVsync),
            ui_scale: SyltUiScaleSetting(1.0),
        }
    }
}

fn handle_settings_loaded(
    mut sylt_signal_reader: EventReader<SyltSignal>,
    mut settings: ResMut<SyltSettings>,
) {
    for event in sylt_signal_reader.read() {
        if let SyltSignal::FileLoaded { key, data } = event {
            debug!("Settings loaded");
            if key.to_string() == SETTINGS_FILE {
                if let Ok(new_settings) =
                    serde_yaml::from_str::<SyltSettings>(data)
                {
                    *settings = new_settings;
                }
            }
        }
    }
}

fn trigger_load_settings(mut sylt_signal_writer: EventWriter<SyltSignal>) {
    sylt_signal_writer.write(SyltSignal::LoadFile {
        key: SETTINGS_FILE.into(),
    });
}

fn trigger_save_settings(
    mut sylt_signal_writer: EventWriter<SyltSignal>,
    settings: Res<SyltSettings>,
    mut settings_event_reader: EventReader<SettingsEvent>,
) {
    for event in settings_event_reader.read() {
        if let SettingsEvent::Save = event {
            sylt_signal_writer.write(SyltSignal::SaveFile {
                key: SETTINGS_FILE.into(),
                data: serde_yaml::to_string(&*settings).unwrap().into(),
            });
        }
    }
}

fn handle_settings_saved(mut sylt_signal_reader: EventReader<SyltSignal>) {
    for event in sylt_signal_reader.read() {
        if let SyltSignal::FileSaved { key } = event {
            if key.to_string() == SETTINGS_FILE {
                debug!("Settings saved");
            }
        }
    }
}
