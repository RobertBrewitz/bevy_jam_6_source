use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
    ui::ContentSize,
};

use crate::{
    canvas::{
        text::{SyltText, SyltTextStyle},
        ui_canvas::{SyltUiScene, SyltUiText},
    },
    i18n::SyltI18nText,
};

#[derive(Component, Default)]
#[require(SyltUiScene, SyltUiText, SyltI18nText, Node, ContentSize)]
pub struct SyltHint {
    pub for_entity: Option<Entity>,
}

pub trait SyltHintSpawnExt {
    fn spawn_sylt_hint(
        &mut self,
        i18n_key: &str,
        for_entity: Entity,
        bundle: impl Bundle,
    ) -> EntityCommands;
}

impl SyltHintSpawnExt for Commands<'_, '_> {
    fn spawn_sylt_hint(
        &mut self,
        i18n_key: &str,
        for_entity: Entity,
        bundle: impl Bundle,
    ) -> EntityCommands {
        let hint = self.spawn((
            SyltHint {
                for_entity: Some(for_entity),
            },
            SyltI18nText::from_key(i18n_key),
            SyltUiText,
            SyltTextStyle {
                font_size: 12.0,
                ..default()
            },
            SyltText::default(),
            RenderLayers::layer(1),
        ));

        let hint_id = hint.id();

        let mut wrapper = self.spawn((
            Name::new("SyltHintWrapper"),
            Node {
                display: Display::Flex,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
        ));

        wrapper.insert(bundle);
        wrapper.add_children(&[hint_id]);
        wrapper
    }
}
