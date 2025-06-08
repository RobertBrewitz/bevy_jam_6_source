use bevy::prelude::*;

pub mod cardinal_navigation;
pub mod components;
pub mod constants;
pub mod copy_paste;
pub mod escape;
pub mod layouts;
pub mod system_set;

#[cfg(feature = "ui_debug")]
mod debug;

pub struct SyltUiPlugin;

impl Plugin for SyltUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            cardinal_navigation::SyltCardinalNavigationPlugin,
            components::SyltUiComponentPlugin,
            escape::SyltEscapePlugin,
            system_set::UiSystemSetPlugin,
            copy_paste::SyltCopyPastePlugin,
        ));

        #[cfg(feature = "ui_debug")]
        app.add_plugins(debug::SyltUiDebugPlugin);
    }
}
