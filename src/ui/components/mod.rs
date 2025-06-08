use bevy::prelude::*;

pub mod button;
pub mod focus_animation;
pub mod glyph;
pub mod hint;
pub mod input;
pub mod label;
pub mod panel;
pub mod select;
pub mod slider;

pub struct SyltUiComponentPlugin;

impl Plugin for SyltUiComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            button::SyltButtonPlugin,
            input::SyltInputPlugin,
            panel::SyltUiPanelPlugin,
            select::SyltSelectPlugin,
            focus_animation::FocusAnimationPlugin,
            slider::SyltSliderPlugin,
        ));
    }
}
