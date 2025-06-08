use bevy::prelude::*;

use crate::{routes::SyltRouterState, sounds::SyltSoundAssets};

pub struct SyltLoadingRoutePlugin;

impl Plugin for SyltLoadingRoutePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            assets_loaded_system.run_if(in_state(SyltRouterState::Loading)),
        );
    }
}

fn assets_loaded_system(
    mut state: ResMut<NextState<SyltRouterState>>,
    // svg_handles: Res<SyltSvgCollection>,
    asset_server: Res<AssetServer>,
    sound_assets: Res<SyltSoundAssets>,
) {
    if !asset_server.is_loaded(&sound_assets.noop) {
        return;
    }

    if !asset_server.is_loaded(&sound_assets.music_loop) {
        return;
    }

    state.set(SyltRouterState::Splash);
}
