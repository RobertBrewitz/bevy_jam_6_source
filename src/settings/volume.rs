use bevy::{audio::Volume, prelude::*};

use crate::{
    routes::SyltRouterState,
    settings::SyltSettings,
    sounds::{SyltMusic, SyltSoundEffect, SyltUiSound},
    ui::components::slider::SyltSlider,
};

use super::SettingsEvent;

pub struct SyltVolumePlugin;

impl Plugin for SyltVolumePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_settings_update);
        app.add_systems(
            Update,
            (
                music_volume_system,
                master_volume_system,
                ui_volume_system,
                effects_volume_system,
            )
                .run_if(in_state(SyltRouterState::Settings)),
        );
    }
}

fn on_settings_update(
    settings: ResMut<SyltSettings>,
    mut music_q: Query<
        &mut AudioSink,
        (
            With<SyltMusic>,
            Without<SyltUiSound>,
            Without<SyltEffectsVolume>,
        ),
    >,
    mut ui_q: Query<
        &mut AudioSink,
        (
            With<SyltUiSound>,
            Without<SyltMusic>,
            Without<SyltSoundEffect>,
        ),
    >,
    mut effects_q: Query<
        &mut AudioSink,
        (
            With<SyltSoundEffect>,
            Without<SyltUiSound>,
            Without<SyltMusic>,
        ),
    >,
) {
    if settings.is_changed() {
        for mut audio_sink in music_q.iter_mut() {
            let new_volume = settings.music_volume * settings.master_volume;

            if audio_sink.volume() == Volume::Linear(new_volume) {
                continue;
            }

            audio_sink.set_volume(Volume::Linear(new_volume));
        }

        for mut audio_sink in ui_q.iter_mut() {
            let new_volume = settings.ui_volume * settings.master_volume;

            if audio_sink.volume() == Volume::Linear(new_volume) {
                continue;
            }

            audio_sink.set_volume(Volume::Linear(new_volume));
        }

        for mut audio_sink in effects_q.iter_mut() {
            let new_volume = settings.effects_volume * settings.master_volume;

            if audio_sink.volume() == Volume::Linear(new_volume) {
                continue;
            }

            audio_sink.set_volume(Volume::Linear(new_volume));
        }
    }
}

#[derive(Component)]
pub struct SyltMasterVolume;

fn master_volume_system(
    slider_q: Query<&SyltSlider, (With<SyltMasterVolume>, Changed<SyltSlider>)>,
    mut settings: ResMut<SyltSettings>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for slider in slider_q.iter() {
        if settings.master_volume == slider.to {
            continue;
        }

        settings.master_volume = slider.to;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

#[derive(Component)]
pub struct SyltMusicVolume;

fn music_volume_system(
    slider_q: Query<&SyltSlider, (With<SyltMusicVolume>, Changed<SyltSlider>)>,
    mut settings: ResMut<SyltSettings>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for slider in slider_q.iter() {
        if settings.music_volume == slider.to {
            continue;
        }

        settings.music_volume = slider.to;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

#[derive(Component)]
pub struct SyltUiVolume;

fn ui_volume_system(
    slider_q: Query<&SyltSlider, (With<SyltUiVolume>, Changed<SyltSlider>)>,
    mut settings: ResMut<SyltSettings>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for slider in slider_q.iter() {
        if settings.ui_volume == slider.to {
            continue;
        }

        settings.ui_volume = slider.to;
        settings_event_writer.write(SettingsEvent::Save);
    }
}

#[derive(Component)]
pub struct SyltEffectsVolume;

fn effects_volume_system(
    slider_q: Query<
        &SyltSlider,
        (With<SyltEffectsVolume>, Changed<SyltSlider>),
    >,
    mut settings: ResMut<SyltSettings>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for slider in slider_q.iter() {
        if settings.effects_volume == slider.to {
            continue;
        }

        settings.effects_volume = slider.to;
        settings_event_writer.write(SettingsEvent::Save);
    }
}
