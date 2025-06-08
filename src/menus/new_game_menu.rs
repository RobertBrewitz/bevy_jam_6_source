use bevy::prelude::*;

use crate::{
    routes::SyltRouterState,
    ui::{
        cardinal_navigation::{
            CardinalCrosshairExt, SyltCardinalFocusedEvent,
            SyltCardinalNavigation,
        },
        components::button::{SyltButtonExt, SyltButtonNavigationExt},
        escape::SyltEscape,
        layouts::flex_col_center_center,
    },
};

use super::SyltMenuState;

pub struct SyltNewGameMenuPlugin;

impl Plugin for SyltNewGameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltMenuState::NewGame), spawn_new_game_menu);
    }
}

fn spawn_new_game_menu(
    mut event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut cmd: Commands,
) {
    cmd.spawn_cardinal_crosshair(StateScoped(SyltMenuState::NewGame));

    let wrapper = cmd
        .spawn((
            StateScoped(SyltMenuState::NewGame),
            flex_col_center_center(),
        ))
        .id();

    let start_button_id = cmd
        .spawn_sylt_button("start", ())
        .navigate_on_click(SyltRouterState::Game)
        .id();

    let back_button_id = cmd
        .spawn_sylt_button("back", SyltEscape)
        .navigate_on_click(SyltRouterState::Title)
        .id();

    cmd.entity(start_button_id).insert(SyltCardinalNavigation {
        north: Some(back_button_id),
        south: Some(back_button_id),
        ..Default::default()
    });

    cmd.entity(back_button_id).insert(SyltCardinalNavigation {
        north: Some(start_button_id),
        south: Some(start_button_id),
        ..Default::default()
    });

    cmd.entity(wrapper)
        .add_children(&[start_button_id, back_button_id]);

    event_writer.write(SyltCardinalFocusedEvent(Some(start_button_id)));
}
