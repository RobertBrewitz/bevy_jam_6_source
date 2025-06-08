use bevy::prelude::*;

mod exit_route;
pub mod game_route;
mod loading_route;
mod settings_route;
mod splash_route;
mod title_route;

#[derive(Debug, Default, Clone, States, Hash, PartialEq, Eq, Reflect)]
#[states(scoped_entities)]
pub enum SyltRouterState {
    #[default]
    Loading,
    Splash,
    Title,

    Continue,
    NewGame,
    LoadGame,

    // ---
    // -- Online Play
    // -- Profile
    // -- Login/Logout (confirm)
    // ---
    // - Settings
    // - Exit
    // - Discord Link`
    // - Socials
    PlayOnline,
    // - Profile
    // - Create Account
    Game,
    // - New
    // - Identities
    Settings,
    // - Sound
    // - Gameplay
    // - Video
    // - Controls
    RedeemInviteToken,

    Login,
    Logout,
    Lobbies,
    CreateLobby,
    Lobby,

    Exit,
}

pub struct SyltRoutesPlugin;

impl Plugin for SyltRoutesPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SyltRouterState>();

        app.init_state::<SyltRouterState>();

        app.add_plugins((
            exit_route::SyltExitRoutePlugin,
            game_route::SyltGameRoutePlugin,
            loading_route::SyltLoadingRoutePlugin,
            settings_route::SyltSettingsRoutePlugin,
            splash_route::SyltSplashRoutePlugin,
            title_route::SyltTitleRoutePlugin,
        ));
    }
}
