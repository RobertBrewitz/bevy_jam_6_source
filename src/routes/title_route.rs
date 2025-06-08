use bevy::prelude::*;

use crate::{menus::SyltMenuState, routes::SyltRouterState};

pub struct SyltTitleRoutePlugin;

impl Plugin for SyltTitleRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Title), open_title_menu);
        app.add_systems(OnExit(SyltRouterState::Title), close_title_menu);
    }
}

fn open_title_menu(mut menu_state: ResMut<NextState<SyltMenuState>>) {
    menu_state.set(SyltMenuState::Title);
}

fn close_title_menu(mut menu_state: ResMut<NextState<SyltMenuState>>) {
    menu_state.set(SyltMenuState::None);
}
