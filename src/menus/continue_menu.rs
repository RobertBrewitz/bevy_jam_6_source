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

pub struct SyltContinueMenuPlugin;

impl Plugin for SyltContinueMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltMenuState::Continue), spawn_load_game_menu);
    }
}

fn spawn_load_game_menu(
    mut event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut cmd: Commands,
) {
    cmd.spawn_cardinal_crosshair(StateScoped(SyltMenuState::Continue));

    let wrapper = cmd
        .spawn((
            StateScoped(SyltMenuState::Continue),
            flex_col_center_center(),
        ))
        .id();

    let continue_button_id = cmd
        .spawn_sylt_button("ready", ())
        .navigate_on_click(SyltRouterState::Game)
        .id();

    let back_button_id = cmd
        .spawn_sylt_button("back", SyltEscape)
        .navigate_on_click(SyltRouterState::Title)
        .id();

    cmd.entity(continue_button_id)
        .insert(SyltCardinalNavigation {
            north: Some(back_button_id),
            south: Some(back_button_id),
            ..Default::default()
        });

    cmd.entity(back_button_id).insert(SyltCardinalNavigation {
        north: Some(continue_button_id),
        south: Some(continue_button_id),
        ..Default::default()
    });

    cmd.entity(wrapper)
        .add_children(&[continue_button_id, back_button_id]);

    event_writer.write(SyltCardinalFocusedEvent(Some(continue_button_id)));
}
