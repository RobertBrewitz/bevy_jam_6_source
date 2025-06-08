use bevy::{app::AppExit, prelude::*};

use super::SyltRouterState;

pub struct SyltExitRoutePlugin;

impl Plugin for SyltExitRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Exit), exit_system);
    }
}

fn exit_system(mut exit: EventWriter<AppExit>) {
    #[cfg(not(target_family = "wasm"))]
    exit.write(AppExit::Success);
}
