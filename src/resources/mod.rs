use bevy::prelude::*;

pub mod token_store;

pub struct SyltResourcePlugin;

impl Plugin for SyltResourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<token_store::TokenStore>();
    }
}
