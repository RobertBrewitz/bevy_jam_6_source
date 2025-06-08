use bevy::prelude::*;

use crate::{
    game::system_set::SyltGamePauseState,
    menus::SyltMenuState,
    routes::SyltRouterState,
    ui::{
        cardinal_navigation::{
            SyltCardinalFocusedEvent, SyltCardinalNavigation,
        },
        components::button::{
            SyltButtonExt, SyltButtonNavigationExt, SyltButtonPressed,
        },
    },
};

pub struct SyltPauseMenuPlugin;

impl Plugin for SyltPauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltMenuState::Pause), spawn_pause_menu);
    }
}

fn spawn_pause_menu(
    mut cmd: Commands,
    mut event_writer: EventWriter<SyltCardinalFocusedEvent>,
) {
    let container_id = cmd
        .spawn((
            StateScoped(SyltMenuState::Pause),
            Name::new("Pause Menu Container"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .id();

    let resume_button = cmd
        .spawn_sylt_button("resume", ())
        .observe(
            |_trigger: Trigger<SyltButtonPressed>,
             mut menu_state: ResMut<NextState<SyltMenuState>>,
             mut pause_state: ResMut<NextState<SyltGamePauseState>>| {
                pause_state.set(SyltGamePauseState(false));
                menu_state.set(SyltMenuState::None);
            },
        )
        .observe(
            |_trigger: Trigger<Pointer<Released>>,
             mut menu_state: ResMut<NextState<SyltMenuState>>,
             mut pause_state: ResMut<NextState<SyltGamePauseState>>| {
                pause_state.set(SyltGamePauseState(false));
                menu_state.set(SyltMenuState::None);
            },
        )
        .id();

    event_writer.write(SyltCardinalFocusedEvent(Some(resume_button)));

    let main_menu_button = cmd
        .spawn_sylt_button("title_menu", ())
        .navigate_on_click(SyltRouterState::Title)
        .id();

    cmd.entity(resume_button).insert(SyltCardinalNavigation {
        north: Some(main_menu_button),
        south: Some(main_menu_button),
        ..Default::default()
    });

    cmd.entity(main_menu_button).insert(SyltCardinalNavigation {
        north: Some(resume_button),
        south: Some(resume_button),
        ..Default::default()
    });

    cmd.entity(container_id)
        .add_children(&[resume_button, main_menu_button]);
}
