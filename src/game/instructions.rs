use bevy::{prelude::*, render::view::RenderLayers, ui::ContentSize};

use crate::{
    canvas::{
        text::{SyltText, SyltTextAlign, SyltTextStyle},
        ui_canvas::SyltUiText,
    },
    i18n::SyltI18nText,
    routes::SyltRouterState,
    ui::constants::SU4,
};

pub struct InstructionsPlugin;

impl Plugin for InstructionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InstructionState>();

        app.add_systems(
            OnEnter(SyltRouterState::Game),
            (spawn_wrapper, show_instructions).chain(),
        );

        app.add_systems(
            OnExit(SyltRouterState::Game),
            (hide_instructions).chain(),
        );

        app.add_systems(
            OnEnter(InstructionState::Build),
            spawn_build_instructions
                .run_if(resource_exists::<InstructionsWrapper>),
        );

        app.add_systems(
            OnEnter(InstructionState::Gameplay),
            spawn_gameplay_instructions
                .run_if(resource_exists::<InstructionsWrapper>),
        );
    }
}

fn show_instructions(
    mut instruction_state: ResMut<NextState<InstructionState>>,
) {
    instruction_state.set(InstructionState::Gameplay);
}

fn hide_instructions(
    mut instruction_state: ResMut<NextState<InstructionState>>,
) {
    instruction_state.set(InstructionState::None);
}

#[derive(Resource, Deref)]
pub struct InstructionsWrapper(pub Entity);

fn spawn_wrapper(
    mut cmd: Commands,
    mut instruction_state: ResMut<NextState<InstructionState>>,
) {
    let wrapper = cmd
        .spawn((
            StateScoped(SyltRouterState::Game),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                row_gap: Val::Px(SU4),
                column_gap: Val::Px(SU4),
                padding: UiRect {
                    bottom: Val::Px(SU4),
                    ..default()
                },
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    cmd.insert_resource(InstructionsWrapper(wrapper));
    instruction_state.set(InstructionState::Gameplay);
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum InstructionState {
    #[default]
    None,
    Gameplay,
    Build,
}

fn spawn_build_instructions(
    mut cmd: Commands,
    wrapper: Res<InstructionsWrapper>,
) {
    let child = cmd
        .spawn((
            StateScoped(InstructionState::Build),
            RenderLayers::layer(1),
            SyltUiText,
            SyltTextStyle {
                font_size: 28.,
                ..default()
            },
            SyltTextAlign::Middle,
            SyltText::default(),
            SyltI18nText::from_key("build instructions"),
            Node::default(),
            ContentSize::default(),
        ))
        .id();

    cmd.entity(**wrapper).add_child(child);
}

fn spawn_gameplay_instructions(
    mut cmd: Commands,
    wrapper: Res<InstructionsWrapper>,
) {
    let child = cmd
        .spawn((
            StateScoped(InstructionState::Gameplay),
            RenderLayers::layer(1),
            SyltUiText,
            SyltTextStyle {
                font_size: 28.,
                ..default()
            },
            SyltTextAlign::Middle,
            SyltText::default(),
            SyltI18nText::from_key("gameplay instructions"),
            Node::default(),
            ContentSize::default(),
        ))
        .id();

    cmd.entity(**wrapper).add_child(child);
}
