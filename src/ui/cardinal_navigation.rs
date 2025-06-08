use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::view::RenderLayers,
    window::PrimaryWindow,
};

use crate::{
    canvas::ui_canvas::{NoSyltUiScaling, SyltUiScene},
    settings::SyltSettings,
    sounds::{play_ui_sound_despawn, SyltSoundAssets},
    vectors::{
        easings::{ease_out_bounce, ease_out_elastic_f64},
        rectangle::SyltRectExt,
    },
};

use super::{
    components::input::SyltInsertModeResource, system_set::SyltUiSystem,
};

pub struct SyltCardinalNavigationPlugin;

impl Plugin for SyltCardinalNavigationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SyltCardinalNavigation>()
            .register_type::<SyltCardinalFocusedResource>()
            .register_type::<CardinalCrosshair>()
            .insert_resource(SyltCardinalFocusedResource(None))
            .add_event::<SyltCardinalFocusedEvent>()
            .add_observer(cardinal_focus_trigger::<Pointer<Over>>())
            .add_systems(
                Update,
                (focus_north, focus_east, focus_south, focus_west)
                    .run_if(not(resource_exists::<SyltInsertModeResource>)),
            )
            .add_systems(
                Update,
                update_cardinal_focused_resource
                    .chain()
                    .in_set(SyltUiSystem::Layout), // .run_if(resource_exists::<SyltSoundAssets>),
            )
            .add_systems(
                Update,
                (
                    initial_crosshair_position,
                    reset_crosshair,
                    tick_timers,
                    lerp_crosshair_to_focused_entity,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                draw_crosshair.in_set(SyltUiSystem::CanvasForeground),
            );
    }
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct SyltCardinalFocusedResource(pub Option<Entity>);

#[derive(Event)]
pub struct SyltCardinalFocusedEvent(pub Option<Entity>);

#[derive(Component, Default)]
pub struct SyltCardinalFocusable;

#[derive(Component, Default, Reflect)]
pub struct SyltCardinalNavigation {
    pub north: Option<Entity>,
    pub south: Option<Entity>,
    pub east: Option<Entity>,
    pub west: Option<Entity>,
}

fn initial_crosshair_position(
    mut focus_crosshair_q: Query<&mut Transform, Added<CardinalCrosshair>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for mut cardinal_crosshair in focus_crosshair_q.iter_mut() {
        cardinal_crosshair.translation =
            Vec3::new(window.width() / 2., -window.height() / 2., 500.);
    }
}

fn update_cardinal_focused_resource(
    mut cmd: Commands,
    mut cardinal_focused_resource: ResMut<SyltCardinalFocusedResource>,
    mut cardinal_focused_events: EventReader<SyltCardinalFocusedEvent>,
    sounds: Res<SyltSoundAssets>,
    settings: Res<SyltSettings>,
) {
    for event in cardinal_focused_events.read() {
        if event.0 == cardinal_focused_resource.0 {
            continue;
        }

        if cardinal_focused_resource.0.is_some() && event.0.is_some() {
            cmd.spawn(play_ui_sound_despawn(sounds.noop.clone(), &settings));
        }

        cardinal_focused_resource.0 = event.0;
    }
}

fn cardinal_focus_trigger<E: std::fmt::Debug + Clone + Reflect>() -> impl Fn(
    Trigger<E>,
    EventWriter<SyltCardinalFocusedEvent>,
    Query<&SyltCardinalFocusable>,
) {
    move |trigger, mut event_writer, focusable_q| {
        if focusable_q.get(trigger.target()).is_ok() {
            event_writer
                .write(SyltCardinalFocusedEvent(Some(trigger.target())));
        }
    }
}

pub fn focus_north(
    entities_with_cardinal_navigation: Query<(Entity, &SyltCardinalNavigation)>,
    focused_entity_resource: Res<SyltCardinalFocusedResource>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
    keyboard_input_resource: Res<ButtonInput<KeyCode>>,
) {
    let shift = keyboard_input_resource
        .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    let mut tab = false;
    let mut up = false;

    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::Tab => tab = true,
                KeyCode::ArrowUp => up = true,
                KeyCode::KeyW => up = true,
                KeyCode::KeyK => up = true,
                _ => {}
            }
        }
    }

    if (shift && tab) || up {
        for (entity, nav) in entities_with_cardinal_navigation.iter() {
            if Some(entity) != focused_entity_resource.0 {
                continue;
            }

            if let Some(entity) = nav.north {
                cardinal_focus_event_writer
                    .write(SyltCardinalFocusedEvent(Some(entity)));
            }
        }
    }
}

fn focus_east(
    entities_with_cardinal_navigation: Query<(Entity, &SyltCardinalNavigation)>,
    focused_entity_resource: Res<SyltCardinalFocusedResource>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
) {
    let mut right = false;

    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::ArrowRight => right = true,
                KeyCode::KeyD => right = true,
                KeyCode::KeyL => right = true,
                _ => {}
            }
        }
    }

    if right {
        for (entity, nav) in entities_with_cardinal_navigation.iter() {
            if Some(entity) != focused_entity_resource.0 {
                continue;
            }

            if let Some(entity) = nav.east {
                cardinal_focus_event_writer
                    .write(SyltCardinalFocusedEvent(Some(entity)));
            }
        }
    }
}

fn focus_south(
    entities_with_cardinal_navigation: Query<(Entity, &SyltCardinalNavigation)>,
    focused_entity_resource: Res<SyltCardinalFocusedResource>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
    keyboard_input_resource: Res<ButtonInput<KeyCode>>,
) {
    let shift = keyboard_input_resource
        .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    let mut tab = false;
    let mut down = false;

    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::Tab => tab = true,
                KeyCode::ArrowDown => down = true,
                KeyCode::KeyS => down = true,
                KeyCode::KeyJ => down = true,
                _ => {}
            }
        }
    }

    if (!shift && tab) || down {
        for (entity, nav) in entities_with_cardinal_navigation.iter() {
            if Some(entity) != focused_entity_resource.0 {
                continue;
            }

            if let Some(entity) = nav.south {
                cardinal_focus_event_writer
                    .write(SyltCardinalFocusedEvent(Some(entity)));
            }
        }
    }
}

fn focus_west(
    entities_with_cardinal_navigation: Query<(Entity, &SyltCardinalNavigation)>,
    focused_entity_resource: Res<SyltCardinalFocusedResource>,
    mut cardinal_focus_event_writer: EventWriter<SyltCardinalFocusedEvent>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
) {
    let mut left = false;

    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::ArrowLeft => left = true,
                KeyCode::KeyA => left = true,
                KeyCode::KeyH => left = true,
                _ => {}
            }
        }
    }

    if left {
        for (entity, nav) in entities_with_cardinal_navigation.iter() {
            if Some(entity) != focused_entity_resource.0 {
                continue;
            }

            if let Some(entity) = nav.west {
                cardinal_focus_event_writer
                    .write(SyltCardinalFocusedEvent(Some(entity)));
            }
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(SyltUiScene)]
pub struct CardinalCrosshair {
    from_size: Vec2,
    to_size: Vec2,
    size: Vec2,
    cooldown_timer: Timer,
    size_timer: Timer,
    move_timer: Timer,
    move_from: Vec3,
    move_to: Vec3,
}

pub trait CardinalCrosshairExt {
    fn spawn_cardinal_crosshair(&mut self, bundle: impl Bundle) -> Entity;
}

impl CardinalCrosshairExt for Commands<'_, '_> {
    fn spawn_cardinal_crosshair(&mut self, bundle: impl Bundle) -> Entity {
        self.spawn((
            bundle,
            CardinalCrosshair::default(),
            RenderLayers::layer(1),
            NoSyltUiScaling,
        ))
        .id()
    }
}

impl Default for CardinalCrosshair {
    fn default() -> Self {
        Self {
            from_size: Vec2::ZERO,
            to_size: Vec2::ZERO,
            size: Vec2::ZERO,
            cooldown_timer: Timer::from_seconds(0.2, TimerMode::Once),
            size_timer: Timer::from_seconds(0.3, TimerMode::Once),
            move_timer: Timer::from_seconds(0.3, TimerMode::Once),
            move_from: Vec3::ZERO,
            move_to: Vec3::ZERO,
        }
    }
}

fn draw_crosshair(
    mut focus_crosshair_q: Query<(&CardinalCrosshair, &mut SyltUiScene)>,
) {
    for (focus_crosshair, mut scene) in focus_crosshair_q.iter_mut() {
        let size = focus_crosshair.size;
        let scene = &mut scene.inner;
        scene.reset();

        let boundry_shape =
            vello::kurbo::Rect::new(0., 0., size.x as f64, size.y as f64);

        let padding = 4.0.lerp(
            8.,
            ease_out_elastic_f64(focus_crosshair.move_timer.fraction() as f64),
        );

        scene.stroke(
            &vello::kurbo::Stroke {
                width: 2.0,
                ..default()
            },
            vello::kurbo::Affine::translate((
                -size.x as f64 / 2.,
                -size.y as f64 / 2.,
            )),
            vello::peniko::Color::new([0.8, 0.6, 0.2, 1.0]),
            None,
            &boundry_shape.plot_rect_corners(8.0, padding),
        );
    }
}

fn reset_crosshair(
    mut focus_crosshair_q: Query<(&mut CardinalCrosshair, &Transform)>,
    focusable_q: Query<
        (Entity, &GlobalTransform, &ComputedNode),
        With<SyltCardinalFocusable>,
    >,
    focused_entity_resource: Res<SyltCardinalFocusedResource>,
) {
    for (mut focus_crosshair, crosshair_gt) in focus_crosshair_q.iter_mut() {
        for (entity, target_gt, node) in focusable_q.iter() {
            if Some(entity) == focused_entity_resource.0 {
                if focused_entity_resource.is_changed() {
                    //focus_crosshair.cooldown_timer.reset();
                    focus_crosshair.move_timer.reset();
                    focus_crosshair.size_timer.reset();
                    focus_crosshair.move_from = crosshair_gt.translation;
                    focus_crosshair.from_size = focus_crosshair.size;
                }

                focus_crosshair.move_to = target_gt.translation();
                focus_crosshair.to_size = node.size();
            }
        }
    }
}

fn tick_timers(
    mut focus_crosshair_q: Query<&mut CardinalCrosshair>,
    time: Res<Time>,
) {
    for mut focus_crosshair in focus_crosshair_q.iter_mut() {
        //focus_crosshair.cooldown_timer.tick(time.delta());

        //if focus_crosshair.cooldown_timer.finished() {
        focus_crosshair.size_timer.tick(time.delta());
        focus_crosshair.move_timer.tick(time.delta());
        //}

        //if focus_crosshair.from_size.x > focus_crosshair.to_size.x {
        //    if focus_crosshair.move_timer.finished()
        //        && focus_crosshair.cooldown_timer.finished()
        //    {
        //        focus_crosshair.size_timer.tick(time.delta());
        //    }

        //    focus_crosshair.move_timer.tick(time.delta());
        //} else {
        //    if focus_crosshair.size_timer.finished() {
        //        focus_crosshair.move_timer.tick(time.delta());
        //    }

        //    if focus_crosshair.cooldown_timer.finished() {
        //        focus_crosshair.size_timer.tick(time.delta());
        //    }
        //}
    }
}

fn lerp_crosshair_to_focused_entity(
    mut focus_crosshair_q: Query<(&mut CardinalCrosshair, &mut Transform)>,
    mut focused_entity_q: Query<Entity, With<SyltCardinalFocusable>>,
    cardinal_focused_resource: Res<SyltCardinalFocusedResource>,
    //sound_q: Query<&AudioSink, With<SyltUiSound>>,
) {
    for (mut focus_crosshair, mut transform) in focus_crosshair_q.iter_mut() {
        let move_t = focus_crosshair.move_timer.fraction();
        let size_t = focus_crosshair.size_timer.fraction();
        for focused_entity in focused_entity_q.iter_mut() {
            if cardinal_focused_resource.0 == Some(focused_entity) {
                focus_crosshair.size = focus_crosshair
                    .from_size
                    .lerp(focus_crosshair.to_size, ease_out_bounce(size_t));
                transform.translation = focus_crosshair
                    .move_from
                    .lerp(focus_crosshair.move_to, ease_out_bounce(move_t));
                transform.translation.z = 500.;
            }
        }
    }
}
