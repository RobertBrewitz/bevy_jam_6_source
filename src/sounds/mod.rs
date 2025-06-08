use bevy::{audio::Volume, prelude::*};

use crate::settings::SyltSettings;

pub struct SyltSoundsPlugin;

impl Plugin for SyltSoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_sounds);
    }
}

#[derive(Resource)]
pub struct SyltSoundAssets {
    pub noop: Handle<AudioSource>,
    pub music_loop: Handle<AudioSource>,
    pub pulsate: Handle<AudioSource>,
    pub click: Handle<AudioSource>,
}

pub fn load_sounds(asset_server: Res<AssetServer>, mut cmd: Commands) {
    cmd.insert_resource(SyltSoundAssets {
        noop: asset_server.load("Kenney/Interface Sounds/Audio/error_001.ogg"),
        music_loop: asset_server
            .load("Kenney/Music Loops/Retro/Retro Reggae.ogg"),
        pulsate: asset_server.load("Kenney/Retro Sounds 2/Audio/upgrade1.ogg"),
        click: asset_server.load("Kenney/Retro Sounds 2/Audio/coin5.ogg"),
    });
}

#[derive(Component)]
pub struct SyltSoundEffect;

pub fn play_game_sound_despawn(
    handle: Handle<AudioSource>,
    settings: &Res<SyltSettings>,
) -> impl Bundle {
    let volume = settings.ui_volume * settings.master_volume;

    (
        SyltSoundEffect,
        AudioPlayer(handle),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            mode: bevy::audio::PlaybackMode::Despawn,
            ..Default::default()
        },
    )
}

#[derive(Component)]
pub struct SyltUiSound;

pub fn play_ui_sound_despawn(
    handle: Handle<AudioSource>,
    settings: &Res<SyltSettings>,
) -> impl Bundle {
    let volume = settings.ui_volume * settings.master_volume;

    (
        SyltUiSound,
        AudioPlayer(handle),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            mode: bevy::audio::PlaybackMode::Despawn,
            ..Default::default()
        },
    )
}

#[derive(Component)]
pub struct SyltMusic;

pub fn loop_music(
    handle: Handle<AudioSource>,
    settings: &Res<SyltSettings>,
) -> impl Bundle {
    let volume = settings.ui_volume * settings.master_volume;

    (
        SyltMusic,
        AudioPlayer(handle),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            mode: bevy::audio::PlaybackMode::Loop,
            ..Default::default()
        },
    )
}
