use std::ops::{Deref, DerefMut};

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

use crate::{
    routes::SyltRouterState,
    settings::{
        language::SyltLanguage,
        video::{SyltMsaa, SyltUiScale, SyltVerticalSync, SyltWindowMode},
        volume::{
            SyltEffectsVolume, SyltMasterVolume, SyltMusicVolume, SyltUiVolume,
        },
        SyltSettings,
    },
    ui::{
        cardinal_navigation::{
            CardinalCrosshairExt, SyltCardinalFocusable,
            SyltCardinalFocusedEvent, SyltCardinalNavigation,
        },
        components::{
            button::{SyltButtonExt, SyltButtonNavigationExt},
            label::SyltLabelSpawnExt,
            select::{SyltOption, SyltSelectExt},
            slider::SyltSliderExt,
        },
        constants::{SU4, SU8},
        escape::SyltEscape,
        layouts::{horizontal_center_layout, label_layout},
    },
};

pub struct SyltSettingsRoutePlugin;

impl Plugin for SyltSettingsRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Settings), setup_layout)
            .add_sub_state::<SettingsState>()
            .add_systems(
                OnEnter(SettingsState::Gameplay),
                setup_gameplay_settings,
            )
            .add_systems(OnEnter(SettingsState::Audio), setup_audio_settings)
            .add_systems(OnEnter(SettingsState::Video), setup_video_settings)
            .add_systems(
                OnEnter(SettingsState::Controls),
                setup_controls_settings,
            )
            .add_systems(
                Update,
                (settings_route_navigation, change_state_system)
                    .run_if(in_state(SyltRouterState::Settings)),
            );
    }
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(SyltRouterState = SyltRouterState::Settings)]
#[states(scoped_entities)]
pub enum SettingsState {
    #[default]
    Gameplay,
    Audio,
    Video,
    Controls,
}

#[derive(Component)]
struct GameplaySettingsLabel;

fn setup_gameplay_settings(
    mut cmd: Commands,
    settings: Res<SyltSettings>,
    container: Single<Entity, With<SettingsContentContainer>>,
    mut gameplay_label: Single<
        (Entity, &mut SyltCardinalNavigation),
        With<GameplaySettingsLabel>,
    >,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    let (entity, gameplay_label_cardinal) = gameplay_label.deref_mut();

    let language_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Gameplay)))
        .id();

    let language_label_id = cmd.spawn_sylt_label("language_select", ());

    let language_select = cmd
        .spawn_sylt_select(
            vec![
                SyltOption {
                    i18n_key: "en".to_string(),
                },
                SyltOption {
                    i18n_key: "es".to_string(),
                },
                SyltOption {
                    i18n_key: "sv".to_string(),
                },
                SyltOption {
                    i18n_key: "pl".to_string(),
                },
            ],
            settings.locale.into(),
            SyltLanguage,
            (),
        )
        .id();

    gameplay_label_cardinal.north = Some(language_select);
    gameplay_label_cardinal.south = Some(language_select);

    cmd.entity(language_select).insert(SyltCardinalNavigation {
        north: Some(*entity),
        south: Some(*entity),
        ..default()
    });

    cmd.entity(language_wrapper)
        .add_children(&[language_label_id, language_select]);

    cmd.entity(*container).add_children(&[language_wrapper]);

    cardinal_focus_event_writer.write(SyltCardinalFocusedEvent(Some(*entity)));
}

#[derive(Component)]
struct AudioSettingsLabel;

fn setup_audio_settings(
    mut cmd: Commands,
    settings: Res<SyltSettings>,
    content: Single<Entity, With<SettingsContentContainer>>,
    mut audio_label_q: Single<
        (Entity, &mut SyltCardinalNavigation),
        With<AudioSettingsLabel>,
    >,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    let content = content.deref();
    let (entity, audio_label) = audio_label_q.deref_mut();

    // master volume
    let master_volume_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Audio)))
        .id();
    let master_volume_label_id = cmd.spawn_sylt_label("master_volume", ());
    let master_volume_slider = cmd
        .spawn_sylt_slider(
            0.0..=1.0,
            settings.master_volume,
            (),
            SyltMasterVolume,
        )
        .id();
    cmd.entity(master_volume_wrapper)
        .add_children(&[master_volume_label_id, master_volume_slider]);

    // music volum
    let music_volume_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Audio)))
        .id();
    let music_volume_label_id = cmd.spawn_sylt_label("music_volume", ());
    let music_volume_slider = cmd
        .spawn_sylt_slider(
            0.0..=1.0,
            settings.music_volume,
            (),
            SyltMusicVolume,
        )
        .id();
    cmd.entity(music_volume_wrapper)
        .add_children(&[music_volume_label_id, music_volume_slider]);

    // ui volum
    let ui_volume_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Audio)))
        .id();
    let ui_volume_label_id = cmd.spawn_sylt_label("ui_volume", ());
    let ui_volume_slider = cmd
        .spawn_sylt_slider(0.0..=1.0, settings.ui_volume, (), SyltUiVolume)
        .id();
    cmd.entity(ui_volume_wrapper)
        .add_children(&[ui_volume_label_id, ui_volume_slider]);

    // sound effects
    let sound_effects_volume_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Audio)))
        .id();
    let sound_effects_volume_label_id =
        cmd.spawn_sylt_label("sound_effects_volume", ());
    let sound_effects_volume_slider = cmd
        .spawn_sylt_slider(
            0.0..=1.0,
            settings.effects_volume,
            (),
            SyltEffectsVolume,
        )
        .id();

    cmd.entity(sound_effects_volume_wrapper).add_children(&[
        sound_effects_volume_label_id,
        sound_effects_volume_slider,
    ]);

    cmd.entity(*content).add_children(&[
        master_volume_wrapper,
        music_volume_wrapper,
        ui_volume_wrapper,
        sound_effects_volume_wrapper,
    ]);

    audio_label.north = Some(ui_volume_slider);
    audio_label.south = Some(master_volume_slider);

    cmd.entity(master_volume_slider)
        .insert(SyltCardinalNavigation {
            north: Some(*entity),
            south: Some(music_volume_slider),
            ..default()
        });

    cmd.entity(music_volume_slider)
        .insert(SyltCardinalNavigation {
            north: Some(master_volume_slider),
            south: Some(ui_volume_slider),
            ..default()
        });

    cmd.entity(ui_volume_slider).insert(SyltCardinalNavigation {
        north: Some(music_volume_slider),
        south: Some(sound_effects_volume_slider),
        ..default()
    });

    cmd.entity(sound_effects_volume_slider)
        .insert(SyltCardinalNavigation {
            north: Some(ui_volume_slider),
            south: Some(*entity),
            ..default()
        });

    cardinal_focus_event_writer.write(SyltCardinalFocusedEvent(Some(*entity)));
}

#[derive(Component)]
struct VideoSettingsLabel;

fn setup_video_settings(
    mut cmd: Commands,
    settings: Res<SyltSettings>,
    content: Single<Entity, With<SettingsContentContainer>>,
    mut video_label: Single<
        (Entity, &mut SyltCardinalNavigation),
        With<VideoSettingsLabel>,
    >,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    let content = content.deref();
    let (video_settings_label, video_label) = video_label.deref_mut();

    let vsync_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Video)))
        .id();
    let vsync_label = cmd.spawn_sylt_label("vsync_select", ());
    let vsync_select = cmd
        .spawn_sylt_select(
            vec![
                SyltOption {
                    i18n_key: "vsync_off".to_string(),
                },
                SyltOption {
                    i18n_key: "vsync_on".to_string(),
                },
            ],
            settings.vsync.into(),
            SyltVerticalSync,
            (),
        )
        .id();

    let ui_scale_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Video)))
        .id();
    let ui_scale_label = cmd.spawn_sylt_label("ui_scale_select", ());
    let ui_scale_select = cmd
        .spawn_sylt_select(
            vec![
                SyltOption {
                    i18n_key: "ui_scale_0.25x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_0.5x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_0.75x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_1x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_1.5x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_2x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_3x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_4x".to_string(),
                },
                SyltOption {
                    i18n_key: "ui_scale_5x".to_string(),
                },
            ],
            settings.ui_scale.into(),
            SyltUiScale,
            (),
        )
        .id();
    cmd.entity(ui_scale_wrapper)
        .add_children(&[ui_scale_label, ui_scale_select]);

    cmd.entity(vsync_wrapper)
        .add_children(&[vsync_label, vsync_select]);

    let msaa_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Video)))
        .id();
    let msaa_label = cmd.spawn_sylt_label("msaa_select", ());
    let msaa_select = cmd
        .spawn_sylt_select(
            vec![
                SyltOption {
                    i18n_key: "msaa_off".to_string(),
                },
                SyltOption {
                    i18n_key: "msaa_2x".to_string(),
                },
                SyltOption {
                    i18n_key: "msaa_4x".to_string(),
                },
                SyltOption {
                    i18n_key: "msaa_8x".to_string(),
                },
            ],
            settings.msaa_setting.into(),
            SyltMsaa,
            (),
        )
        .id();
    cmd.entity(msaa_wrapper)
        .add_children(&[msaa_label, msaa_select]);

    let window_mode_wrapper = cmd
        .spawn((label_layout(), StateScoped(SettingsState::Video)))
        .id();
    let window_mode_label = cmd.spawn_sylt_label("window_mode_select", ());
    let window_mode_select = cmd
        .spawn_sylt_select(
            vec![
                SyltOption {
                    i18n_key: "window".to_string(),
                },
                SyltOption {
                    i18n_key: "borderless_fullscreen".to_string(),
                },
            ],
            settings.window_mode.into(),
            SyltWindowMode,
            (),
        )
        .id();
    cmd.entity(window_mode_wrapper)
        .add_children(&[window_mode_label, window_mode_select]);

    cmd.entity(*content).add_children(&[
        vsync_wrapper,
        msaa_wrapper,
        window_mode_wrapper,
        ui_scale_wrapper,
    ]);

    video_label.north = Some(ui_scale_select);
    video_label.south = Some(vsync_select);

    cmd.entity(vsync_select).insert(SyltCardinalNavigation {
        north: Some(*video_settings_label),
        south: Some(msaa_select),
        ..default()
    });

    cmd.entity(msaa_select).insert(SyltCardinalNavigation {
        north: Some(vsync_select),
        south: Some(window_mode_select),
        ..default()
    });

    cmd.entity(window_mode_select)
        .insert(SyltCardinalNavigation {
            north: Some(msaa_select),
            south: Some(ui_scale_select),
            ..default()
        });

    cmd.entity(ui_scale_select).insert(SyltCardinalNavigation {
        north: Some(window_mode_select),
        south: Some(*video_settings_label),
        ..default()
    });

    cardinal_focus_event_writer
        .write(SyltCardinalFocusedEvent(Some(*video_settings_label)));
}

#[derive(Component)]
struct ControlsSettingsLabel;

#[derive(Component)]
struct ChangeSettingsState(SettingsState);

fn setup_controls_settings(
    controls_label: Single<Entity, With<ControlsSettingsLabel>>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    cardinal_focus_event_writer
        .write(SyltCardinalFocusedEvent(Some(*controls_label)));
}

#[derive(Component)]
struct SettingsContentContainer;

fn setup_layout(
    mut cmd: Commands,
    mut next_route: ResMut<NextState<SettingsState>>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    let layout = cmd
        .spawn((
            Name::new("Settings Container"),
            StateScoped(SyltRouterState::Settings),
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0)],
                grid_template_rows: vec![
                    GridTrack::max_content(),
                    GridTrack::flex(1.0),
                    GridTrack::max_content(),
                ],
                row_gap: Val::Px(SU4),
                column_gap: Val::Px(SU4),
                padding: UiRect {
                    left: Val::Px(SU8),
                    right: Val::Px(SU8),
                    top: Val::Px(SU8),
                    bottom: Val::Px(SU8),
                },
                ..default()
            },
        ))
        .id();

    let header_container = cmd
        .spawn((
            Name::new("Settings Header Container"),
            horizontal_center_layout(),
        ))
        .id();

    let content_container = cmd
        .spawn((
            Name::new("Settings Content Container"),
            SettingsContentContainer,
            Node {
                display: Display::Flex,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                flex_grow: 0.0,
                padding: UiRect {
                    left: Val::Px(100.0),
                    right: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let footer_container = cmd
        .spawn((
            Name::new("Settings Footer Container"),
            Node {
                display: Display::Flex,
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    let gameplay_menu_label_id = cmd.spawn_sylt_label(
        "gameplay_settings",
        (GameplaySettingsLabel, SyltCardinalFocusable),
    );

    let audio_menu_label_id = cmd.spawn_sylt_label(
        "audio_settings",
        (AudioSettingsLabel, SyltCardinalFocusable),
    );

    let video_menu_label_id = cmd.spawn_sylt_label(
        "video_settings",
        (VideoSettingsLabel, SyltCardinalFocusable),
    );

    // let controls_menu_label_id = cmd.spawn_sylt_label(
    //     "controls_settings",
    //     (ControlsSettingsLabel, SyltCardinalFocusable),
    // );

    let back_button = cmd
        .spawn_sylt_button("back esc", SyltEscape)
        .navigate_on_click(SyltRouterState::Title)
        .id();

    cmd.entity(gameplay_menu_label_id).insert((
        SyltCardinalNavigation {
            west: Some(video_menu_label_id),
            east: Some(audio_menu_label_id),
            ..default()
        },
        ChangeSettingsState(SettingsState::Gameplay),
    ));

    cmd.entity(audio_menu_label_id).insert((
        SyltCardinalNavigation {
            west: Some(gameplay_menu_label_id),
            east: Some(video_menu_label_id),
            ..default()
        },
        ChangeSettingsState(SettingsState::Audio),
    ));

    cmd.entity(video_menu_label_id).insert((
        SyltCardinalNavigation {
            west: Some(audio_menu_label_id),
            east: Some(gameplay_menu_label_id),
            ..default()
        },
        ChangeSettingsState(SettingsState::Video),
    ));

    // cmd.entity(controls_menu_label_id).insert((
    //     SyltCardinalNavigation {
    //         west: Some(video_menu_label_id),
    //         east: Some(gameplay_menu_label_id),
    //         ..default()
    //     },
    //     ChangeSettingsState(SettingsState::Controls),
    // ));

    cardinal_focus_event_writer
        .write(SyltCardinalFocusedEvent(Some(gameplay_menu_label_id)));

    cmd.entity(header_container).add_children(&[
        gameplay_menu_label_id,
        audio_menu_label_id,
        video_menu_label_id,
        // controls_menu_label_id,
    ]);

    cmd.entity(footer_container).add_children(&[back_button]);

    cmd.entity(layout).add_children(&[
        header_container,
        content_container,
        footer_container,
    ]);

    cmd.spawn_cardinal_crosshair(StateScoped(SyltRouterState::Settings));

    next_route.set(SettingsState::Gameplay);
}

fn change_state_system(
    mut next_route: ResMut<NextState<SettingsState>>,
    mut cardinal_focused_events: EventReader<SyltCardinalFocusedEvent>,
    change_settings_state_q: Query<(Entity, &ChangeSettingsState)>,
) {
    for event in cardinal_focused_events.read() {
        for (entity, change_settings_state) in change_settings_state_q.iter() {
            if Some(entity) == event.0 {
                next_route.set(change_settings_state.0.clone());
            }
        }
    }
}

fn settings_route_navigation(
    mut next_route: ResMut<NextState<SettingsState>>,
    current: Res<State<SettingsState>>,
    mut key_events: EventReader<KeyboardInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for event in key_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        if keyboard_input.just_pressed(KeyCode::KeyN) {
            let next = match current.get() {
                SettingsState::Gameplay => SettingsState::Audio,
                SettingsState::Audio => SettingsState::Video,
                SettingsState::Video => SettingsState::Gameplay,
                SettingsState::Controls => SettingsState::Gameplay,
            };
            next_route.set(next);
        }

        if keyboard_input.just_pressed(KeyCode::KeyP) {
            let next = match current.get() {
                SettingsState::Gameplay => SettingsState::Controls,
                SettingsState::Audio => SettingsState::Gameplay,
                SettingsState::Video => SettingsState::Audio,
                SettingsState::Controls => SettingsState::Video,
            };
            next_route.set(next);
        }
    }
}
