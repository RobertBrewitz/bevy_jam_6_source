use bevy::prelude::*;

use super::components::{
    button::{SyltButton, SyltButtonPressed},
    input::SyltInsertModeResource,
};

pub struct SyltEscapePlugin;

impl Plugin for SyltEscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            button_on_press_escape_system
                .run_if(not(resource_exists::<SyltInsertModeResource>)),
        );
    }
}

#[derive(Component)]
pub struct SyltEscape;

fn button_on_press_escape_system(
    mut cmd: Commands,
    q: Query<Entity, (With<SyltButton>, With<SyltEscape>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for entity in q.iter() {
            cmd.trigger_targets(SyltButtonPressed, entity);
        }
    }
}
