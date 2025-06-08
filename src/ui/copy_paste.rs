use bevy::prelude::*;
use copypasta::ClipboardContext;

pub struct SyltCopyPastePlugin;

impl Plugin for SyltCopyPastePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SyltClipboard {
            context: ClipboardContext::new().unwrap(),
        });
    }
}

#[derive(Resource)]
pub struct SyltClipboard {
    pub context: ClipboardContext,
}
