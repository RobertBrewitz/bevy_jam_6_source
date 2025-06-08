use crate::{
    canvas::{
        text::{SyltText, SyltTextStyle},
        ui_canvas::{SyltUiScene, SyltUiText},
    },
    i18n::SyltI18nText,
    routes::SyltRouterState,
    ui::{
        cardinal_navigation::{
            SyltCardinalFocusable, SyltCardinalFocusedEvent,
            SyltCardinalFocusedResource,
        },
        constants::{SU2, SU4},
    },
};
use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    ui::ContentSize,
};

use super::{focus_animation::SyltFocusPrimary, input::SyltInsertModeResource};

pub const ROTATION_SPEED: f64 = 0.1;
pub const ANIMATION_SPEED: f64 = 0.4;
pub const PIXELS_PER_SECOND: f64 = 200.;

pub struct SyltButtonPlugin;

impl Plugin for SyltButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SyltButtonPressed>();
        app.add_event::<SyltButtonFocused>();

        app.add_systems(
            Update,
            (
                sylt_button_pressed_trigger,
                sylt_button_cardinal_focus_trigger,
            )
                .run_if(not(resource_exists::<SyltInsertModeResource>)),
        );

        app.add_observer(sylt_button_click_trigger);
        app.add_observer(on_sylt_button_cardinal_focus);
    }
}

#[derive(Event, Clone, Debug, Reflect)]
pub struct SyltButtonFocused;

#[derive(Event, Clone, Debug, Reflect)]
pub struct SyltButtonPressed;

#[derive(Default, Component)]
#[require(
    SyltUiScene,
    SyltUiText,
    SyltFocusPrimary,
    SyltCardinalFocusable,
    Node,
    Interaction,
    ContentSize
)]
pub struct SyltButton;

pub trait SyltButtonExt {
    fn spawn_sylt_button(
        &mut self,
        i18n_key: &str,
        bundle: impl Bundle,
    ) -> EntityCommands;
}

pub trait SyltButtonNavigationExt {
    fn navigate_on_click(&mut self, route: SyltRouterState) -> &mut Self;
}

impl SyltButtonNavigationExt for EntityCommands<'_> {
    fn navigate_on_click(&mut self, route: SyltRouterState) -> &mut Self {
        let route1 = route.clone();
        let route2 = route.clone();

        self.observe(
            move |_: Trigger<SyltButtonPressed>,
                  mut n: ResMut<NextState<SyltRouterState>>| {
                n.set(route1.clone());
            },
        )
        .observe(
            move |_: Trigger<Pointer<Released>>,
                  mut n: ResMut<NextState<SyltRouterState>>| {
                n.set(route2.clone());
            },
        )
    }
}

impl SyltButtonExt for Commands<'_, '_> {
    fn spawn_sylt_button(
        &mut self,
        i18n_key: &str,
        bundle: impl Bundle,
    ) -> EntityCommands {
        let mut button = self.spawn((
            Name::new(format!("{} SyltButton", i18n_key)),
            RenderLayers::layer(1),
            SyltButton,
            Node {
                padding: UiRect::axes(Val::Px(SU4), Val::Px(SU2)),
                ..default()
            },
            SyltUiText,
            SyltText::default(),
            SyltTextStyle {
                brush: vello::peniko::Brush::Solid(vello::peniko::Color::WHITE),
                ..default()
            },
            SyltI18nText::from_key(i18n_key),
        ));
        button.insert(bundle);
        button
    }
}

fn sylt_button_click_trigger(
    trigger: Trigger<Pointer<Released>, SyltButton>,
    mut cmd: Commands,
) {
    cmd.trigger_targets(SyltButtonPressed, trigger.target());
}

fn sylt_button_pressed_trigger(
    mut cmd: Commands,
    q: Query<Entity, With<SyltButton>>,
    cardinal_focused_resource: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for entity in q.iter() {
        if keyboard_input.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
            if let Some(focused_entity) = cardinal_focused_resource.0 {
                if focused_entity == entity {
                    cmd.trigger_targets(SyltButtonPressed, entity);
                }
            }
        }
    }
}

fn on_sylt_button_cardinal_focus(
    trigger: Trigger<SyltCardinalFocusedEvent, SyltButton>,
    mut cmd: Commands,
) {
    cmd.trigger_targets(SyltButtonFocused, trigger.target());
}

fn sylt_button_cardinal_focus_trigger(
    mut cmd: Commands,
    q: Query<Entity, With<SyltButton>>,
    cardinal_focused_resource: Res<SyltCardinalFocusedResource>,
) {
    if cardinal_focused_resource.is_changed() {
        if let Some(focused_entity) = cardinal_focused_resource.0 {
            for entity in q.iter() {
                if focused_entity == entity {
                    cmd.trigger_targets(SyltButtonFocused, entity);
                }
            }
        }
    }
}
