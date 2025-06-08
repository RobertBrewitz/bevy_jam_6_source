use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{
    settings::{volume::SyltUiVolume, SyltSettings},
    ui::cardinal_navigation::SyltCardinalFocusedResource,
};

pub struct SyltUiSoundPlugin;

impl Plugin for SyltUiSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiSoundEvent>();
        app.add_systems(Startup, load_ui_sounds);
        app.add_systems(Update, (on_ui_sound_events, ui_volume_slider_sound));
    }
}

#[derive(Resource)]
pub struct SyltUiSoundAssets {
    pub cardinal_focus: Handle<AudioSource>,
    pub slider_inc: Handle<AudioSource>,
    pub slider_dec: Handle<AudioSource>,
}

pub fn load_ui_sounds(asset_server: Res<AssetServer>, mut cmd: Commands) {
    cmd.insert_resource(SyltUiSoundAssets {
        cardinal_focus: asset_server.load("sounds/ui/cardinal_focus.wav"),
        slider_inc: asset_server.load("sounds/ui/slider.wav"),
        slider_dec: asset_server.load("sounds/ui/slider.wav"),
    });
}

#[derive(Component)]
pub struct SyltUiAudio;

#[derive(Event)]
pub enum UiSoundEvent {
    CardinalFocus,
    SliderInc,
    SliderDec,
}

fn on_ui_sound_events(
    mut cmd: Commands,
    mut event_reader: EventReader<UiSoundEvent>,
    ui_sounds: Res<SyltUiSoundAssets>,
    settings: Res<SyltSettings>,
) {
    for event in event_reader.read() {
        let volume = settings.ui_volume * settings.master_volume;

        match event {
            UiSoundEvent::CardinalFocus => {
                cmd.spawn((
                    AudioPlayer(ui_sounds.cardinal_focus.clone()),
                    PlaybackSettings {
                        volume: Volume::Linear(volume),
                        mode: PlaybackMode::Despawn,
                        ..Default::default()
                    },
                    SyltUiAudio,
                ));
            }
            UiSoundEvent::SliderInc => {
                cmd.spawn((
                    AudioPlayer(ui_sounds.slider_inc.clone()),
                    PlaybackSettings {
                        volume: Volume::Linear(volume),
                        mode: PlaybackMode::Despawn,
                        ..Default::default()
                    },
                    SyltUiAudio,
                ));
            }
            UiSoundEvent::SliderDec => {
                cmd.spawn((
                    AudioPlayer(ui_sounds.slider_dec.clone()),
                    PlaybackSettings {
                        volume: Volume::Linear(volume),
                        mode: PlaybackMode::Despawn,
                        ..Default::default()
                    },
                    SyltUiAudio,
                ));
            }
        }
    }
}

fn ui_volume_slider_sound(
    mut select_q: Query<Entity, With<SyltUiVolume>>,
    current_cardinal_focus: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ui_sound_event: EventWriter<UiSoundEvent>,
) {
    if keyboard_input.any_just_pressed([
        KeyCode::KeyD,
        KeyCode::ArrowRight,
        KeyCode::KeyL,
        KeyCode::KeyA,
        KeyCode::ArrowLeft,
        KeyCode::KeyH,
    ]) {
        //let shift = keyboard_input
        //    .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        //let ctrl = keyboard_input
        //    .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

        if let Some(focused_entity) = current_cardinal_focus.0 {
            for entity in select_q.iter_mut() {
                if entity != focused_entity {
                    continue;
                }
                ui_sound_event.write(UiSoundEvent::SliderInc);
            }
        }
    }
}
