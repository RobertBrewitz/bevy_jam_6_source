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

pub struct LoadGameMenuPlugin;

impl Plugin for LoadGameMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltMenuState::LoadGame), spawn_load_game_menu);
    }
}

fn spawn_load_game_menu(
    mut event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut cmd: Commands,
) {
    cmd.spawn_cardinal_crosshair(StateScoped(SyltRouterState::LoadGame));

    let wrapper = cmd
        .spawn((
            StateScoped(SyltRouterState::LoadGame),
            flex_col_center_center(),
        ))
        .id();

    let load_button_id = cmd
        .spawn_sylt_button("load", ())
        .navigate_on_click(SyltRouterState::Game)
        .id();

    let back_button_id = cmd
        .spawn_sylt_button("back", SyltEscape)
        .navigate_on_click(SyltRouterState::Title)
        .id();

    cmd.entity(load_button_id).insert(SyltCardinalNavigation {
        north: Some(back_button_id),
        south: Some(back_button_id),
        ..Default::default()
    });

    cmd.entity(back_button_id).insert(SyltCardinalNavigation {
        north: Some(load_button_id),
        south: Some(load_button_id),
        ..Default::default()
    });

    cmd.entity(wrapper)
        .add_children(&[load_button_id, back_button_id]);

    event_writer.write(SyltCardinalFocusedEvent(Some(load_button_id)));
}
