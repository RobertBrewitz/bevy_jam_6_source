use bevy::prelude::*;

use crate::{routes::SyltRouterState, ui::components::select::SyltSelect};

use super::{SettingsEvent, SyltSettings};

pub struct SyltLanguagePlugin;

impl Plugin for SyltLanguagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            language_system.run_if(in_state(SyltRouterState::Settings)),
        );
    }
}

#[derive(Component)]
pub struct SyltLanguage;

fn language_system(
    mut settings: ResMut<SyltSettings>,
    select_q: Query<&SyltSelect, (With<SyltLanguage>, Changed<SyltSelect>)>,
    mut settings_event_writer: EventWriter<SettingsEvent>,
) {
    for select in select_q.iter() {
        if settings.locale
            == select.options[select.selected_index]
                .i18n_key
                .as_str()
                .into()
        {
            continue;
        }

        settings.locale = select.options[select.selected_index]
            .i18n_key
            .as_str()
            .into();

        settings_event_writer.write(SettingsEvent::Save);
    }
}
