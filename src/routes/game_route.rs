use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    game::system_set::SyltGamePauseState, menus::SyltMenuState,
    routes::SyltRouterState,
};

pub struct SyltGameRoutePlugin;

impl Plugin for SyltGameRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (pause, open_pause_menu).run_if(
                in_state(SyltRouterState::Game)
                    .and(in_state(SyltMenuState::None))
                    .and(
                        input_just_pressed(KeyCode::KeyP)
                            .or(input_just_pressed(KeyCode::Escape)),
                    ),
            ),
        );

        app.add_systems(
            Update,
            close_pause_menu.run_if(
                in_state(SyltRouterState::Game)
                    .and(not(in_state(SyltMenuState::None)))
                    .and(
                        input_just_pressed(KeyCode::KeyP)
                            .or(input_just_pressed(KeyCode::Escape)),
                    ),
            ),
        );

        app.add_systems(
            OnExit(SyltRouterState::Game),
            (close_pause_menu, unpause),
        );

        app.add_systems(
            OnEnter(SyltMenuState::None),
            unpause.run_if(in_state(SyltRouterState::Game)),
        );
    }
}

fn unpause(mut next_pause: ResMut<NextState<SyltGamePauseState>>) {
    next_pause.set(SyltGamePauseState(false));
}

fn pause(mut next_pause: ResMut<NextState<SyltGamePauseState>>) {
    next_pause.set(SyltGamePauseState(true));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<SyltMenuState>>) {
    next_menu.set(SyltMenuState::Pause);
}

fn close_pause_menu(mut next_menu: ResMut<NextState<SyltMenuState>>) {
    next_menu.set(SyltMenuState::None);
}
