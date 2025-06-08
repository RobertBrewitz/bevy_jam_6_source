use bevy::{
    prelude::*,
    window::{PresentMode, WindowMode},
};
use serde::{Deserialize, Serialize};

use crate::{routes::SyltRouterState, ui::components::select::SyltSelect};

use super::{SettingsEvent, SyltSettings};

pub struct SyltVideoPlugin;

impl Plugin for SyltVideoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_settings_update);
        app.add_systems(
            Update,
            (
                msaa_system,
                window_mode_system,
                vsync_system,
                ui_scale_system,
            )
                .run_if(in_state(SyltRouterState::Settings)),
        );
    }
}

fn on_settings_update(
    mut cmd: Commands,
    settings: ResMut<SyltSettings>,
    mut window_q: Query<&mut Window>,
    mut camera_q: Query<(Entity, &Msaa)>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut window in window_q.iter_mut() {
        if window.mode != settings.window_mode.0 {
            window.mode = settings.window_mode.0;
        }

        if window.present_mode != settings.vsync.0 {
            window.present_mode = settings.vsync.0;
        }
    }

    for (entity, msaa) in camera_q.iter_mut() {
        if *msaa != settings.msaa_setting.0 {
            cmd.entity(entity).insert(settings.msaa_setting.0);
        }
    }
}

fn window_mode_system(
    mut settings: ResMut<SyltSettings>,
    select_q: Query<&SyltSelect, (With<SyltWindowMode>, Changed<SyltSelect>)>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for select in select_q.iter() {
        let selected = select.options[select.selected_index]
            .i18n_key
            .as_str()
            .into();

        if settings.window_mode == selected {
            continue;
        }

        settings.window_mode = selected;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

fn vsync_system(
    mut settings: ResMut<SyltSettings>,
    select_q: Query<&SyltSelect, (With<SyltVerticalSync>, Changed<SyltSelect>)>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for select in select_q.iter() {
        let selected = select.options[select.selected_index]
            .i18n_key
            .as_str()
            .into();
        if settings.vsync == selected {
            continue;
        }

        settings.vsync = selected;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

fn msaa_system(
    mut settings: ResMut<SyltSettings>,
    select_q: Query<&SyltSelect, (With<SyltMsaa>, Changed<SyltSelect>)>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for select in select_q.iter() {
        let selected = select.options[select.selected_index]
            .i18n_key
            .as_str()
            .into();

        if settings.msaa_setting == selected {
            continue;
        }

        settings.msaa_setting = selected;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

fn ui_scale_system(
    mut settings: ResMut<SyltSettings>,
    select_q: Query<&SyltSelect, (With<SyltUiScale>, Changed<SyltSelect>)>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for select in select_q.iter() {
        let selected = select.options[select.selected_index]
            .i18n_key
            .as_str()
            .into();

        if settings.ui_scale == selected {
            continue;
        }

        settings.ui_scale = selected;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

#[derive(Component)]
pub struct SyltMsaa;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SyltMsaaSetting(pub Msaa);

impl serde::Serialize for SyltMsaaSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self.0 {
            Msaa::Off => "msaa_off",
            Msaa::Sample2 => "msaa_2x",
            Msaa::Sample4 => "msaa_4x",
            Msaa::Sample8 => "msaa_8x",
        };

        serde::Serialize::serialize(value, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SyltMsaaSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Ok(match value.as_str() {
            "msaa_off" => Self(Msaa::Off),
            "msaa_2x" => Self(Msaa::Sample2),
            "msaa_4x" => Self(Msaa::Sample4),
            "msaa_8x" => Self(Msaa::Sample8),
            _ => Self(Msaa::Off),
        })
    }
}

impl From<SyltMsaaSetting> for &str {
    fn from(value: SyltMsaaSetting) -> Self {
        match value.0 {
            Msaa::Off => "msaa_off",
            Msaa::Sample2 => "msaa_2x",
            Msaa::Sample4 => "msaa_4x",
            Msaa::Sample8 => "msaa_8x",
        }
    }
}

impl From<&str> for SyltMsaaSetting {
    fn from(value: &str) -> Self {
        match value {
            "msaa_off" => Self(Msaa::Off),
            "msaa_2x" => Self(Msaa::Sample2),
            "msaa_4x" => Self(Msaa::Sample4),
            "msaa_8x" => Self(Msaa::Sample8),
            _ => Self(Msaa::Off),
        }
    }
}

impl From<SyltMsaaSetting> for Msaa {
    fn from(value: SyltMsaaSetting) -> Self {
        value.0
    }
}

#[derive(Component)]
pub struct SyltWindowMode;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SyltWindowModeSetting(pub WindowMode);

impl serde::Serialize for SyltWindowModeSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self.0 {
            WindowMode::Windowed => "windowed",
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary) => {
                "borderless_fullscreen"
            }
            _ => "windowed",
        };

        serde::Serialize::serialize(value, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SyltWindowModeSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Ok(match value.as_str() {
            "windowed" => Self(WindowMode::Windowed),
            "borderless_fullscreen" => Self(WindowMode::BorderlessFullscreen(
                MonitorSelection::Primary,
            )),
            _ => Self(WindowMode::Windowed),
        })
    }
}

impl From<SyltWindowModeSetting> for &str {
    fn from(value: SyltWindowModeSetting) -> Self {
        match value.0 {
            WindowMode::Windowed => "windowed",
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary) => {
                "borderless_fullscreen"
            }
            _ => "windowed",
        }
    }
}

impl From<&str> for SyltWindowModeSetting {
    fn from(value: &str) -> Self {
        match value {
            "windowed" => Self(WindowMode::Windowed),
            "borderless_fullscreen" => Self(WindowMode::BorderlessFullscreen(
                MonitorSelection::Primary,
            )),
            _ => Self(WindowMode::Windowed),
        }
    }
}

#[derive(Component)]
pub struct SyltVerticalSync;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SyltPresentModeSetting(pub PresentMode);

impl serde::Serialize for SyltPresentModeSetting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self.0 {
            PresentMode::AutoNoVsync => "vsync_off",
            PresentMode::AutoVsync => "vsync_on",
            _ => "vsync_off",
        };

        serde::Serialize::serialize(value, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for SyltPresentModeSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Ok(match value.as_str() {
            "vsync_off" => Self(PresentMode::AutoNoVsync),
            "vsync_on" => Self(PresentMode::AutoVsync),
            _ => Self(PresentMode::AutoNoVsync),
        })
    }
}

impl From<SyltPresentModeSetting> for &str {
    fn from(value: SyltPresentModeSetting) -> Self {
        match value.0 {
            PresentMode::AutoNoVsync => "vsync_off",
            PresentMode::AutoVsync => "vsync_on",
            _ => "vsync_off",
        }
    }
}

impl From<&str> for SyltPresentModeSetting {
    fn from(value: &str) -> Self {
        match value {
            "vsync_off" => Self(PresentMode::AutoNoVsync),
            "vsync_on" => Self(PresentMode::AutoVsync),
            _ => Self(PresentMode::AutoNoVsync),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SyltUiScaleSetting(pub f32);

#[derive(Component)]
pub struct SyltUiScale;

impl From<SyltUiScaleSetting> for &str {
    fn from(scale: SyltUiScaleSetting) -> Self {
        match scale.0 {
            0.25 => "ui_scale_0.25x",
            0.5 => "ui_scale_0.5x",
            0.75 => "ui_scale_0.75x",
            1.0 => "ui_scale_1x",
            1.5 => "ui_scale_1.5x",
            2.0 => "ui_scale_2x",
            3.0 => "ui_scale_3x",
            4.0 => "ui_scale_4x",
            5.0 => "ui_scale_5x",
            _ => "ui_scale_1x",
        }
    }
}

impl From<&str> for SyltUiScaleSetting {
    fn from(value: &str) -> Self {
        match value {
            "ui_scale_0.25x" => Self(0.25),
            "ui_scale_0.5x" => Self(0.5),
            "ui_scale_0.75x" => Self(0.75),
            "ui_scale_1x" => Self(1.0),
            "ui_scale_1.5x" => Self(1.5),
            "ui_scale_2x" => Self(2.0),
            "ui_scale_3x" => Self(3.0),
            "ui_scale_4x" => Self(4.0),
            "ui_scale_5x" => Self(5.0),
            _ => Self(1.0),
        }
    }
}
