use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use vello::{
    kurbo::Shape,
    peniko::{color::palette, Brush},
};

use crate::{
    cameras::SyltWorldCamera,
    canvas::world_canvas::{SyltPickingShape, SyltWorldScene},
    game::{
        build::ShowBuildMenu,
        system_set::{SyltGameSystemSet, SyltPausableSystems},
    },
    routes::SyltRouterState,
    vectors::{easings::ease_out_elastic, rectangle::SyltRectExt},
};

pub const GRID_WIDTH: i32 = 9;
pub const GRID_HEIGHT: i32 = 9;
pub const CELL_HEIGHT: f32 = 100.;
pub const CELL_WIDTH: f32 = 100.;
pub const CELL_GAP: f32 = 10.;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(SyltRouterState::Game),
            (insert_grid_position, setup_grid),
        );

        app.add_systems(OnExit(SyltRouterState::Game), remove_grid_position);

        app.add_systems(
            Update,
            (keyboard_input_system)
                .in_set(SyltPausableSystems)
                .in_set(SyltGameSystemSet::Input)
                .run_if(
                    in_state(SyltRouterState::Game)
                        .and(resource_exists::<ShowBuildMenu>),
                ),
        );

        app.add_systems(
            Update,
            (draw_grid, animate_camera_to_grid_position)
                .in_set(SyltPausableSystems)
                .run_if(in_state(SyltRouterState::Game)),
        );

        app.add_systems(
            Update,
            (on_grid_position_changed)
                .in_set(SyltPausableSystems)
                .run_if(
                    in_state(SyltRouterState::Game)
                        .and(resource_exists::<FocusedGridPosition>),
                ),
        );
    }
}

fn keyboard_input_system(
    display_shop: Res<ShowBuildMenu>,
    mut cmd: Commands,
    grid_position: Res<FocusedGridPosition>,
    mut keyboard_event_reader: EventReader<KeyboardInput>,
) {
    if display_shop.0 {
        return;
    }

    let mut new_x = grid_position.x;
    let mut new_y = grid_position.y;

    for event in keyboard_event_reader.read() {
        if event.state == ButtonState::Pressed {
            // Left
            if [KeyCode::KeyA, KeyCode::KeyH, KeyCode::ArrowLeft]
                .contains(&event.key_code)
            {
                new_x = grid_position.x - 1;
            }

            // Right
            if [KeyCode::KeyD, KeyCode::KeyL, KeyCode::ArrowRight]
                .contains(&event.key_code)
            {
                new_x = grid_position.x + 1;
            }

            // Up
            if [KeyCode::KeyW, KeyCode::KeyK, KeyCode::ArrowUp]
                .contains(&event.key_code)
            {
                new_y = grid_position.y - 1;
            }

            // Down
            if [KeyCode::KeyS, KeyCode::KeyJ, KeyCode::ArrowDown]
                .contains(&event.key_code)
            {
                new_y = grid_position.y + 1;
            }
        }
    }

    cmd.insert_resource(FocusedGridPosition {
        x: new_x.clamp(0, 8),
        y: new_y.clamp(0, 8),
    });
}

#[derive(Component)]
struct MoveCameraAnimation {
    pub timer: Timer,
    pub from: Vec3,
    pub to: Vec3,
}

fn insert_grid_position(mut cmd: Commands) {
    cmd.insert_resource(FocusedGridPosition { x: 4, y: 4 });
}

fn on_grid_position_changed(
    mut cmd: Commands,
    grid_position: Res<FocusedGridPosition>,
    world_camera: Single<(Entity, &Transform), With<SyltWorldCamera>>,
) {
    let (camera_id, camera_transform) = *world_camera;

    if grid_position.is_changed() {
        let new_x = grid_position.x as f32 * (CELL_GAP + CELL_WIDTH)
            - (CELL_GAP + CELL_WIDTH) / 2.;
        let new_y = -(grid_position.y as f32) * (CELL_GAP + CELL_HEIGHT)
            - (CELL_GAP + CELL_HEIGHT) / 2.;

        cmd.entity(camera_id).insert(MoveCameraAnimation {
            timer: Timer::from_seconds(0.2, TimerMode::Once),
            from: camera_transform.translation,
            to: Vec2::new(new_x, new_y).extend(0.),
        });
    }
}

fn animate_camera_to_grid_position(
    mut cmd: Commands,
    time: Res<Time>,
    world_camera: Single<
        (Entity, &mut Transform, &mut MoveCameraAnimation),
        With<SyltWorldCamera>,
    >,
) {
    let (camera_id, mut transform, mut animation) = world_camera.into_inner();

    animation.timer.tick(time.delta());

    let v = animation.timer.fraction();

    let x = animation.from.x.lerp(animation.to.x, ease_out_elastic(v));
    let y = animation.from.y.lerp(animation.to.y, ease_out_elastic(v));
    transform.translation = Vec3::new(x, y, 0.);

    if animation.timer.finished() {
        cmd.entity(camera_id).remove::<MoveCameraAnimation>();
    }
}

fn remove_grid_position(mut cmd: Commands) {
    cmd.remove_resource::<FocusedGridPosition>();
}

#[derive(Resource)]
pub struct FocusedGridPosition {
    pub x: i32,
    pub y: i32,
}

impl FocusedGridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
struct GameGrid;

#[derive(Component)]
struct GridCell {
    pub x: i32,
    pub y: i32,
}

fn setup_grid(mut cmd: Commands) {
    for i in 0..GRID_WIDTH {
        for j in 0..GRID_HEIGHT {
            cmd.spawn((
                StateScoped(SyltRouterState::Game),
                GridCell { x: i, y: j },
                SyltWorldScene {
                    pixel_size: Vec2::new(CELL_WIDTH, CELL_HEIGHT),
                    ..default()
                },
                Transform::from_xyz(
                    i as f32 * (CELL_WIDTH + CELL_GAP),
                    -(j as f32) * (CELL_HEIGHT + CELL_GAP),
                    0.,
                ),
            ));
        }
    }
}

fn draw_grid(
    mut cmd: Commands,
    cell_q: Query<(Entity, &mut SyltWorldScene, &GridCell)>,
    grid_position: Res<FocusedGridPosition>,
) {
    for (entity, mut scene, cell) in cell_q {
        let scene = &mut scene.inner;

        scene.reset();

        let shape = vello::kurbo::Rect::new(
            0.,
            0.,
            CELL_WIDTH as f64,
            CELL_HEIGHT as f64,
        );

        // cmd.entity(entity).insert(SyltPickingShape {
        //     inner: shape.to_path(0.1),
        //     ..default()
        // });

        scene.fill(
            vello::peniko::Fill::NonZero,
            Default::default(),
            &Brush::Solid(palette::css::DIM_GRAY),
            None,
            &shape,
        );

        if cell.x == grid_position.x && cell.y == grid_position.y {
            let focus_shape = shape.plot_rect_corners(10., 5.);

            scene.stroke(
                &vello::kurbo::Stroke {
                    width: 3.,
                    ..default()
                },
                Default::default(),
                &Brush::Solid(palette::css::SILVER),
                None,
                &focus_shape,
            );
        }
    }
}
