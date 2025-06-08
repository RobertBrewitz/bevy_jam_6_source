use bevy::{prelude::*, render::view::RenderLayers, ui::ContentSize};

use crate::{
    canvas::{ui_canvas::SyltUiScene, ui_canvas::SyltUiText},
    i18n::SyltI18nText,
};

#[derive(Component)]
#[require(SyltUiScene, SyltUiText, Node, ContentSize)]
pub struct SyltLabel;

pub trait SyltLabelSpawnExt {
    fn spawn_sylt_label(
        &mut self,
        i18n_key: &str,
        bundle: impl Bundle,
    ) -> Entity;
}

impl SyltLabelSpawnExt for Commands<'_, '_> {
    fn spawn_sylt_label(
        &mut self,
        i18n_key: &str,
        bundle: impl Bundle,
    ) -> Entity {
        let label = self.spawn((
            Name::new(format!("SyltLabel {}", i18n_key)),
            SyltI18nText::from_key(i18n_key),
            SyltLabel,
            RenderLayers::layer(1),
        ));

        //label.insert(bundle);
        let label_id = label.id();

        let mut wrapper = self.spawn((
            Name::new(format!("SyltLabelWrapper {}", i18n_key)),
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                width: Val::Percent(100.),
                ..default()
            },
            RenderLayers::layer(1),
        ));

        wrapper.insert(bundle);
        wrapper.add_children(&[label_id]);
        wrapper.id()
    }
}
