use crate::{
    canvas::{
        text::{SyltText, SyltTextAlign, SyltTextStyle},
        ui_canvas::{SyltUiScene, SyltUiText},
    },
    i18n::SyltI18nText,
    ui::{
        cardinal_navigation::{
            SyltCardinalFocusable, SyltCardinalFocusedResource,
        },
        copy_paste::SyltClipboard,
        system_set::SyltUiSystem,
    },
};
use bevy::{
    ecs::system::EntityCommands,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
    render::view::RenderLayers,
};
use copypasta::ClipboardProvider;

use super::{focus_animation::SyltFocusPrimary, hint::SyltHint};

pub const PIXELS_PER_SECOND: f64 = 50.;
pub const CURSOR_BLINK_RATE: f64 = 1.;

pub struct SyltInputPlugin;

impl Plugin for SyltInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SyltOnEnterInsertModeEvent>();
        app.add_event::<SyltOnExitInsertModeEvent>();
        app.add_event::<SyltInputOnChangeEvent>();

        app.add_systems(
            Update,
            (
                input_keyboard_system,
                keyboard_enter_insert_mode,
                exit_insert_mode_on_blur,
                input_clicked_system,
                on_enter_insert_mode,
                on_exit_insert_mode,
                input_copy_paste_system,
            ),
        );

        app.add_systems(
            Update,
            draw_cursor.in_set(SyltUiSystem::CanvasForeground),
        );
    }
}

#[derive(Event, Debug)]
pub struct SyltOnEnterInsertModeEvent(pub Entity);

fn on_enter_insert_mode(
    mut cmd: Commands,
    mut event_reader: EventReader<SyltOnEnterInsertModeEvent>,
    mut entity_q: Query<Entity, (With<SyltInput>, Without<SyltInsertMode>)>,
) {
    for event in event_reader.read() {
        for entity in entity_q.iter_mut() {
            if entity == event.0 {
                cmd.insert_resource(SyltInsertModeResource);
                cmd.entity(event.0).insert(SyltInsertMode);
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct SyltOnExitInsertModeEvent(pub Entity);

fn on_exit_insert_mode(
    mut cmd: Commands,
    mut event_reader: EventReader<SyltOnExitInsertModeEvent>,
    mut entity_q: Query<Entity, (With<SyltInput>, With<SyltInsertMode>)>,
) {
    for event in event_reader.read() {
        for entity in entity_q.iter_mut() {
            if entity == event.0 {
                cmd.remove_resource::<SyltInsertModeResource>();
                cmd.entity(entity).remove::<SyltInsertMode>();
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct SyltInputOnChangeEvent(pub Entity);

#[derive(Component, Default)]
#[require(SyltUiScene, SyltCardinalFocusable, SyltFocusPrimary)]
pub struct SyltInput {
    pub pressed: bool,
}

/// Entity with this component is in insert mode.
#[derive(Component)]
struct SyltInsertMode;

/// If this resource exists, we are in insert mode.
#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct SyltInsertModeResource;

#[derive(Component)]
pub struct SyltInputMarker;

pub trait SyltInputExt {
    fn spawn_sylt_input(
        &mut self,
        value: &str,
        bundle: impl Bundle,
    ) -> EntityCommands;
}

impl SyltInputExt for Commands<'_, '_> {
    fn spawn_sylt_input(
        &mut self,
        value: &str,
        bundle: impl Bundle,
    ) -> EntityCommands {
        let cursor_id = self.spawn(cursor()).id();
        let text_id = self.spawn(text(value)).id();
        let mut input = self.spawn(input());
        input.add_children(&[cursor_id, text_id]);
        input.insert(bundle);
        input
    }
}

fn input() -> impl Bundle {
    (
        SyltInput::default(),
        SyltInputMarker,
        RenderLayers::layer(1),
        Node {
            height: Val::Px(40.),
            width: Val::Percent(100.),
            display: Display::Flex,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        ZIndex(1),
    )
}

fn text(value: &str) -> impl Bundle {
    (
        SyltUiText,
        SyltText {
            content: value.to_string(),
            ..default()
        },
        SyltInputMarker,
        Node::default(),
        ZIndex(2),
        RenderLayers::layer(1),
    )
}

fn cursor() -> impl Bundle {
    (
        SyltInputCursor,
        Node::default(),
        ZIndex(2),
        SyltInputMarker,
        RenderLayers::layer(1),
    )
}

fn keyboard_enter_insert_mode(
    mut event_writer: EventWriter<SyltOnEnterInsertModeEvent>,
    mut hint_q: Query<(&SyltHint, &mut SyltI18nText)>,
    mut input_q: Query<Entity, (Without<SyltInsertMode>, With<SyltInput>)>,
    mut key_events: EventReader<KeyboardInput>,
    cardinal_focus_resource: Res<SyltCardinalFocusedResource>,
) {
    for event in key_events.read() {
        for entity in input_q.iter_mut() {
            if event.state == ButtonState::Released {
                continue;
            }

            if let Some(focus_entity) = cardinal_focus_resource.0 {
                if focus_entity == entity
                    && (event.logical_key == Key::Enter
                        || event.key_code == KeyCode::KeyI)
                {
                    for (hint, mut hint_i18n_text) in hint_q.iter_mut() {
                        if Some(entity) == hint.for_entity {
                            hint_i18n_text.update_key("exit insert mode");
                        }
                    }

                    event_writer.write(SyltOnEnterInsertModeEvent(entity));
                }
            }
        }
    }
}

fn exit_insert_mode_on_blur(
    mut event_writer: EventWriter<SyltOnExitInsertModeEvent>,
    mut hint_q: Query<(&SyltHint, &mut SyltI18nText)>,
    mut input_q: Query<Entity, (With<SyltInsertMode>, With<SyltInput>)>,
    cardinal_focus_resource: ResMut<SyltCardinalFocusedResource>,
) {
    for entity in input_q.iter_mut() {
        if let Some(focus_entity) = cardinal_focus_resource.0 {
            if focus_entity == entity {
                continue;
            }

            for (hint, mut hint_i18n_text) in hint_q.iter_mut() {
                if Some(entity) == hint.for_entity {
                    hint_i18n_text.update_key("enter insert mode");
                }
            }

            event_writer.write(SyltOnExitInsertModeEvent(entity));
        }
    }
}

fn input_keyboard_system(
    mut on_exit_event_writer: EventWriter<SyltOnExitInsertModeEvent>,
    mut on_change_event_writer: EventWriter<SyltInputOnChangeEvent>,
    mut key_events: EventReader<KeyboardInput>,
    cardinal_focus_resource: Res<SyltCardinalFocusedResource>,
    mut hint_q: Query<(&SyltHint, &mut SyltI18nText)>,

    mut input_q: Query<
        (Entity, &Children),
        (With<SyltInput>, With<SyltInsertMode>),
    >,
    mut text_q: Query<&mut SyltText, (With<SyltInputMarker>, With<SyltUiText>)>,
) {
    for event in key_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        for (input_entity, children) in input_q.iter_mut() {
            if let Some(focus_entity) = cardinal_focus_resource.0 {
                if focus_entity == input_entity {
                    for child in children {
                        if let Ok(mut sylt_text) = text_q.get_mut(*child) {
                            match &event.logical_key {
                                Key::Character(c) => {
                                    sylt_text.content.push_str(c);
                                    on_change_event_writer.write(
                                        SyltInputOnChangeEvent(input_entity),
                                    );
                                    continue;
                                }
                                Key::Backspace => {
                                    sylt_text.content.pop();
                                    on_change_event_writer.write(
                                        SyltInputOnChangeEvent(input_entity),
                                    );
                                    continue;
                                }
                                Key::Escape | Key::Enter => {
                                    for (hint, mut hint_i18n_text) in
                                        hint_q.iter_mut()
                                    {
                                        if Some(input_entity) == hint.for_entity
                                        {
                                            hint_i18n_text.update_key(
                                                "enter insert mode",
                                            );
                                        }
                                    }

                                    on_exit_event_writer.write(
                                        SyltOnExitInsertModeEvent(input_entity),
                                    );
                                }
                                Key::Space => {
                                    sylt_text.content.push(' ');
                                    on_change_event_writer.write(
                                        SyltInputOnChangeEvent(input_entity),
                                    );
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn input_clicked_system(
    mut q: Query<(Entity, &Interaction, &mut SyltInput), Changed<Interaction>>,
    mut hint_q: Query<(&SyltHint, &mut SyltI18nText)>,
    mut event_writer: EventWriter<SyltOnEnterInsertModeEvent>,
) {
    for (entity, interaction, mut input) in q.iter_mut() {
        if *interaction == Interaction::Pressed {
            input.pressed = true;
        }

        if *interaction == Interaction::Hovered && input.pressed {
            input.pressed = false;
            for (hint, mut hint_i18n_text) in hint_q.iter_mut() {
                if Some(entity) == hint.for_entity {
                    hint_i18n_text.update_key("exit insert mode");
                }
            }

            event_writer.write(SyltOnEnterInsertModeEvent(entity));
        }

        if *interaction == Interaction::None {
            input.pressed = false;
        }
    }
}

fn input_copy_paste_system(
    cardinal_focus_resource: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<SyltClipboard>,

    mut input_q: Query<(Entity, &Children), Without<SyltInsertMode>>,
    mut text_q: Query<&mut SyltText, (With<SyltInputMarker>, With<SyltUiText>)>,
) {
    for (entity, children) in input_q.iter_mut() {
        if let Some(focus_entity) = cardinal_focus_resource.0 {
            if focus_entity != entity {
                continue;
            }

            for child in children {
                if let Ok(mut sylt_text) = text_q.get_mut(*child) {
                    if keyboard_input.any_pressed([
                        KeyCode::ControlLeft,
                        KeyCode::ControlRight,
                    ]) {
                        if keyboard_input.just_pressed(KeyCode::KeyC) {
                            clipboard
                                .context
                                .set_contents(sylt_text.content.clone())
                                .unwrap();
                        }

                        if keyboard_input.just_pressed(KeyCode::KeyV) {
                            if let Ok(clipboard_contents) =
                                clipboard.context.get_contents()
                            {
                                sylt_text.content = clipboard_contents;
                            }
                        }
                    }

                    if keyboard_input.just_pressed(KeyCode::KeyY) {
                        clipboard
                            .context
                            .set_contents(sylt_text.content.clone())
                            .unwrap();
                    }

                    if keyboard_input.just_pressed(KeyCode::KeyP) {
                        if let Ok(clipboard_contents) =
                            clipboard.context.get_contents()
                        {
                            sylt_text.content = clipboard_contents;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Default)]
#[require(SyltUiScene)]
pub struct SyltInputCursor;

fn draw_cursor(
    time: Res<Time>,

    mut input_q: Query<
        (Entity, &Children),
        (Without<SyltInputCursor>, With<SyltInsertMode>),
    >,
    text_q: Query<
        (&SyltText, &SyltTextStyle, &SyltTextAlign),
        (With<SyltInputMarker>, With<SyltUiText>),
    >,
    mut cursor_q: Query<(&ChildOf, &mut SyltUiScene), With<SyltInputCursor>>,
) {
    let visible = (time.elapsed_secs_f64() / CURSOR_BLINK_RATE).fract() < 0.5;

    for (input_entity, children) in input_q.iter_mut() {
        for (cursor_parent, mut scene) in cursor_q.iter_mut() {
            let scene = &mut scene.inner;
            scene.reset();

            if cursor_parent.parent() != input_entity {
                continue;
            }

            for child in children {
                if let Ok((sylt_text, text_style, text_align)) =
                    text_q.get(*child)
                {
                    let size = sylt_text.sizeof(text_style, text_align);
                    let char_size = SyltText {
                        content: "a".to_string(),
                        ..default()
                    }
                    .sizeof(text_style, text_align);

                    if visible {
                        scene.fill(
                            vello::peniko::Fill::NonZero,
                            vello::kurbo::Affine::translate((
                                size.x.abs() as f64,
                                (size.y.abs() / 2.) as f64,
                            )),
                            vello::peniko::Color::new([1., 1., 1., 0.5]),
                            None,
                            &vello::kurbo::Rect::new(
                                0.,
                                0.,
                                char_size.x as f64,
                                char_size.y as f64,
                            ),
                        );
                    }
                }
            }
        }
    }
}
