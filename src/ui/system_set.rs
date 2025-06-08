use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SyltUiSystem {
    Content,
    Layout,
    CanvasBackground,
    CanvasForeground,
}

pub struct UiSystemSetPlugin;

impl Plugin for UiSystemSetPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                SyltUiSystem::Content,
                SyltUiSystem::Layout,
                SyltUiSystem::CanvasForeground,
            )
                .chain(),
        );
    }
}
