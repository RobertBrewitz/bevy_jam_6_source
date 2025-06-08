use bevy::{prelude::*, render::view::RenderLayers, ui::ContentSize};

use crate::{
    canvas::{text::SyltText, ui_canvas::SyltUiText},
    i18n::SyltI18nText,
    routes::SyltRouterState,
    settings::SyltSettings,
    signals::SyltSignal,
    sounds::{loop_music, SyltSoundAssets},
};

pub mod system_set;
use system_set::SyltPausableSystems;

mod build;
mod grid;
mod instructions;
mod nodes;

pub struct SyltGamePlugin;

impl Plugin for SyltGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(system_set::SyltSystemSetPlugin);

        // Add game specific plugins here
        app.add_plugins((
            grid::GridPlugin,
            nodes::NodesPlugin,
            build::BuildPlugin,
            instructions::InstructionsPlugin,
        ));

        app.insert_resource(Sparks(0.0));

        app.add_systems(
            OnEnter(SyltRouterState::Game),
            (init_game_state, spawn_score),
        );

        app.add_systems(
            Update,
            (update_sparks_text)
                .in_set(SyltPausableSystems)
                .run_if(in_state(SyltRouterState::Game)),
        );

        app.add_systems(OnEnter(SyltRouterState::Splash), start_game_music);
    }
}

fn start_game_music(
    mut cmd: Commands,
    sounds: Res<SyltSoundAssets>,
    settings: Res<SyltSettings>,
) {
    cmd.spawn(loop_music(sounds.music_loop.clone(), &settings));
}

#[derive(Resource)]
struct Sparks(pub f32);

#[derive(Component)]
struct SparkNodeButton;

// fn spawn_shop(mut cmd: Commands) {
//     let wrapper = cmd
//         .spawn((
//             StateScoped(SyltRouterState::Game),
//             Node {
//                 position_type: PositionType::Absolute,
//                 right: Val::Px(0.),
//                 top: Val::Px(0.),
//                 width: Val::Percent(20.),
//                 height: Val::Percent(100.),
//                 display: Display::Flex,
//                 flex_direction: FlexDirection::Column,
//                 justify_content: JustifyContent::Center,
//                 align_items: AlignItems::Center,
//                 ..default()
//             },
//         ))
//         .id();
//
//     let spark_node = cmd
//         .spawn_sylt_button("spark node", (SparkNodeButton, ZIndex(10)))
//         .id();
//
//     cmd.entity(wrapper).add_child(spark_node);
// }

// fn on_spark_node_button_pressed(trigger: Trigger<SyltButtonPressed>) {
//     //
//     println!("pressed");
// }

fn spawn_score(mut cmd: Commands) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::Flex,
            top: bevy::ui::Val::Px(10.),
            left: bevy::ui::Val::Px(10.),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|parent| {
        parent.spawn((
            RenderLayers::layer(1),
            StateScoped(SyltRouterState::Game),
            Node { ..default() },
            ContentSize::default(),
            SyltUiText,
            SyltText::default(),
            SyltI18nText::from_key("sparks").add_template_key("spark_count"),
        ));

        parent.spawn((
            RenderLayers::layer(1),
            StateScoped(SyltRouterState::Game),
            Node { ..default() },
            ContentSize::default(),
            SyltUiText,
            SyltText::default(),
            SparksText,
        ));
    });
}

fn init_game_state(mut cmd: Commands) {
    cmd.insert_resource(Sparks(0.0));
}

#[derive(Component)]
struct SparksText;

fn update_sparks_text(
    mut txt: Single<&mut SyltText, With<SparksText>>,
    sparks: Res<Sparks>,
) {
    txt.content = sparks.0.to_string();
}
