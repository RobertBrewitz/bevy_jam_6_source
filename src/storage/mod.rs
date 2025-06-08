use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

// #[cfg(target_arch = "wasm32")]
// pub mod wasm;

pub struct SyltStorageWrapperPlugin;

impl Plugin for SyltStorageWrapperPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(native::SyltStoragePlugin);

        // #[cfg(target_arch = "wasm32")]
        // app.add_plugins(wasm::SyltStoragePlugin);
    }
}
