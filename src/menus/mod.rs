use bevy::prelude::*;

mod continue_menu;
mod load_game_menu;
mod new_game_menu;
mod pause_menu;
mod title_menu;

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum SyltMenuState {
    #[default]
    None,
    Disabled,
    Pause,
    Title,
    NewGame,
    LoadGame,
    Continue,
}

pub struct SyltMenusPlugin;

impl Plugin for SyltMenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SyltMenuState>();

        app.add_plugins((
            pause_menu::SyltPauseMenuPlugin,
            title_menu::TitleMenuPlugin,
            new_game_menu::SyltNewGameMenuPlugin,
            load_game_menu::LoadGameMenuPlugin,
            continue_menu::SyltContinueMenuPlugin,
        ));
    }
}
