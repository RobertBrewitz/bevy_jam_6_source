use std::sync::Arc;

use bevy::prelude::*;

pub struct SyltSignalsPlugin;

impl Plugin for SyltSignalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SyltSignal>();
        app.add_systems(Update, handle_signals);
    }
}

fn handle_signals(mut signals: EventReader<SyltSignal>) {
    for signal in signals.read() {
        debug!("{:?}", signal);
    }
}

#[derive(Debug, Event, Clone)]
pub enum SyltSignal {
    // Messages
    // Filesystem/LocalStorage
    SaveFile { key: Arc<str>, data: Arc<str> },
    FileSaved { key: Arc<str> },
    SaveFileError { key: Arc<str>, message: Arc<str> },
    LoadFile { key: Arc<str> },
    FileLoaded { key: Arc<str>, data: Arc<str> },
    LoadFileError { key: Arc<str>, message: Arc<str> },
}
