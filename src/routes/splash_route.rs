use bevy::{
    input::keyboard::KeyboardInput, prelude::*, render::view::RenderLayers,
    ui::ContentSize,
};

use crate::{
    canvas::{
        text::{SyltText, SyltTextStyle},
        ui_canvas::SyltUiText,
    },
    routes::SyltRouterState,
};

const SPLASH_DURATION: f32 = 1.;

pub struct SyltSplashRoutePlugin;

impl Plugin for SyltSplashRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Splash), splash_enter_system);
        app.add_systems(
            Update,
            (splash_update_system, skip_on_press)
                .run_if(in_state(SyltRouterState::Splash)),
        );
    }
}

#[derive(Component)]
struct SplashMarker;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

fn splash_enter_system(mut commands: Commands) {
    commands
        .spawn((
            StateScoped(SyltRouterState::Splash),
            Name::new("Splash Container"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(32.0),
                ..default()
            },
            SplashMarker,
        ))
        .with_children(|parent| {
            parent.spawn((
                RenderLayers::layer(1),
                SyltUiText,
                SyltTextStyle {
                    font_size: 75.,
                    ..default()
                },
                SyltText {
                    content: env!("CARGO_PKG_NAME").to_string(),
                    ..default()
                },
                Node::default(),
                ContentSize::default(),
            ));

            parent.spawn((
                RenderLayers::layer(1),
                SyltUiText,
                SyltText {
                    content: env!("CARGO_PKG_DESCRIPTION").to_string(),
                    ..default()
                },
                SyltTextStyle {
                    font_size: 50.,
                    ..default()
                },
                Node::default(),
                ContentSize::default(),
            ));

            parent.spawn((
                RenderLayers::layer(1),
                SyltUiText,
                SyltText {
                    content: env!("CARGO_PKG_VERSION").to_string(),
                    ..default()
                },
                SyltTextStyle {
                    font_size: 25.,
                    ..default()
                },
                Node::default(),
                ContentSize::default(),
            ));
        });

    commands.insert_resource(SplashTimer(Timer::from_seconds(
        SPLASH_DURATION,
        TimerMode::Once,
    )));
}

fn splash_update_system(
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
    mut end_state: ResMut<NextState<SyltRouterState>>,
) {
    if timer.tick(time.delta()).just_finished() {
        end_state.set(SyltRouterState::Title);
    }
}

fn skip_on_press(
    mut next_state: ResMut<NextState<SyltRouterState>>,
    mut keyboard_events: EventReader<KeyboardInput>,
) {
    for _ in keyboard_events.read() {
        next_state.set(SyltRouterState::Title);
    }
}
