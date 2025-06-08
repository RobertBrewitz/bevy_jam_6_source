use bevy::{
    ecs::system::{EntityCommands, SystemId},
    prelude::*,
    render::view::RenderLayers,
    ui::ContentSize,
};

use crate::{
    canvas::{ui_canvas::SyltUiScene, ui_canvas::SyltUiText},
    i18n::SyltI18nText,
    ui::{
        cardinal_navigation::{
            SyltCardinalFocusable, SyltCardinalFocusedResource,
        },
        components::focus_animation::SyltFocusPrimary,
    },
};

pub struct SyltSelectPlugin;

impl Plugin for SyltSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SyltSelectUpdateEvent>();

        app.add_systems(
            Update,
            (
                on_spawn_select,
                on_sylt_select_update_event_system,
                select_prev_system,
                select_next_system,
            ),
        );
    }
}

#[derive(Event)]
pub struct SyltSelectUpdateEvent {
    pub entity: Entity,
    pub index: usize,
}

fn on_sylt_select_update_event_system(
    select_q: Query<(Entity, &SyltSelect)>,
    mut text_q: Query<(&ChildOf, &mut SyltI18nText), With<SyltSelectText>>,
) {
    for (entity, select) in select_q.iter() {
        for (parent, mut label) in text_q.iter_mut() {
            if parent.parent() == entity {
                label.update_key(
                    &select.options[select.selected_index].i18n_key,
                );
            }
        }
    }
}

fn on_spawn_select(
    mut select_q: Query<(Entity, &SyltSelect), Added<SyltSelect>>,
    mut text_q: Query<(&ChildOf, &mut SyltI18nText), With<SyltSelectText>>,
) {
    for (entity, select) in select_q.iter_mut() {
        for (parent, mut label) in text_q.iter_mut() {
            if parent.parent() == entity {
                if let Some(option) = &select.options.first() {
                    label.update_key(&option.i18n_key);
                }
            }
        }
    }
}

fn select_prev_system(
    mut select_q: Query<(Entity, &mut SyltSelect)>,
    current_cardinal_focus: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyA)
        || keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::KeyH)
    {
        if let Some(focused_entity) = current_cardinal_focus.0 {
            for (entity, mut select) in select_q.iter_mut() {
                if entity != focused_entity {
                    continue;
                }

                select.selected_index =
                    (select.selected_index + select.options.len() - 1)
                        % select.options.len();
            }
        }
    }
}

fn select_next_system(
    mut select_q: Query<(Entity, &mut SyltSelect)>,
    current_cardinal_focus: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyD)
        || keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::KeyL)
    {
        if let Some(focused_entity) = current_cardinal_focus.0 {
            for (entity, mut select) in select_q.iter_mut() {
                if entity != focused_entity {
                    continue;
                }

                select.selected_index =
                    (select.selected_index + 1) % select.options.len();
            }
        }
    }
}

pub struct SyltOption {
    pub i18n_key: String,
}

#[derive(Default, Component)]
#[require(
    SyltUiScene,
    SyltCardinalFocusable,
    SyltFocusPrimary,
    Node,
    ContentSize
)]
pub struct SyltSelect {
    pub options: Vec<SyltOption>,
    pub selected_index: usize,
    pub system_id: Option<SystemId>,
}

pub trait SyltSelectExt {
    fn spawn_sylt_select(
        &mut self,
        options: Vec<SyltOption>,
        selected_key: &str,
        wrapper_bundle: impl Bundle,
        bundle: impl Bundle,
    ) -> EntityCommands;
}

impl SyltSelectExt for Commands<'_, '_> {
    fn spawn_sylt_select(
        &mut self,
        options: Vec<SyltOption>,
        selected_key: &str,
        wrapper_bundle: impl Bundle,
        bundle: impl Bundle,
    ) -> EntityCommands {
        let mut text = self.spawn(selected_option_text(selected_key));
        let text_id = text.id();
        text.insert(bundle);

        let mut wrapper = self.spawn(select(options, selected_key));
        wrapper.add_children(&[text_id]);
        wrapper.insert(wrapper_bundle);
        wrapper
    }
}

#[derive(Component)]
pub struct SyltSelectText;

fn select(options: Vec<SyltOption>, selected_key: &str) -> impl Bundle {
    let selected_index = options
        .iter()
        .position(|option| option.i18n_key == selected_key)
        .unwrap_or(0);

    (
        Name::new(format!("SyltSelect {}", selected_key)),
        SyltSelect {
            options,
            selected_index,
            ..default()
        },
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.),
            ..default()
        },
        RenderLayers::layer(1),
    )
}

#[derive(Component)]
#[require(Node, SyltUiText, ContentSize, SyltUiScene)]
struct SyltSelectSelectedOption;

fn selected_option_text(selected_key: &str) -> impl Bundle {
    (
        Name::new(format!("SyltSelectSelectedText {}", selected_key)),
        SyltI18nText::from_key(selected_key),
        SyltSelectSelectedOption,
        SyltSelectText,
        ZIndex(10),
        RenderLayers::layer(1),
    )
}
