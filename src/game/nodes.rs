use bevy::prelude::*;
use vello::kurbo::Shape;

use crate::{
    canvas::world_canvas::{
        SyltPickingShape, SyltWorldCanvasScaleFactor, SyltWorldScene,
    },
    game::{
        build::ShowBuildMenu,
        grid::{
            FocusedGridPosition, GridPosition, CELL_GAP, CELL_HEIGHT,
            CELL_WIDTH,
        },
        system_set::{SyltGameSystemSet, SyltPausableSystems},
        Sparks,
    },
    routes::SyltRouterState,
    settings::SyltSettings,
    sounds::{play_game_sound_despawn, SyltSoundAssets},
    vectors::{bumps::bump_logistic, polygon::plot_polygon_path},
};

pub struct NodesPlugin;

impl Plugin for NodesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Game), place_initial_node);

        app.add_systems(
            Update,
            (on_keyboard_press)
                .in_set(SyltPausableSystems)
                .in_set(SyltGameSystemSet::Input)
                .run_if(
                    in_state(SyltRouterState::Game)
                        .and(resource_exists::<ShowBuildMenu>),
                ),
        );

        app.add_systems(
            Update,
            (
                draw_stimulator_node,
                draw_clicker_node,
                position_node,
                pulsate_stimulator_node,
                animate_stimulator_node,
                animate_clicker_node,
            )
                .in_set(SyltPausableSystems)
                .in_set(SyltGameSystemSet::Update)
                .run_if(in_state(SyltRouterState::Game)),
        );

        app.add_observer(on_pointer_release);
    }
}

#[derive(Component, Default)]
pub struct SparkNode {
    pub original_translation: Vec3,
}

pub const CLICKER_NODE_COST: f32 = 10.;
#[derive(Component, Default)]
#[require(SyltWorldScene)]
pub struct ClickerNode {
    pub animation_timer: Timer,
}

pub const STIMULATOR_NODE_COST: f32 = 20.;
#[derive(Component, Default)]
#[require(SyltWorldScene)]
pub struct StimulatorNode {
    pub animation_timer: Timer,
    pub click_timer: Timer,
}

fn place_initial_node(mut cmd: Commands) {
    cmd.spawn((
        StateScoped(SyltRouterState::Game),
        SparkNode::default(),
        ClickerNode::default(),
        SyltWorldCanvasScaleFactor(1.0),
        GridPosition::new(4, 4),
        Transform::from_xyz(0., 0., 100.),
    ));
}

/// positions nodes when GridPosition changes
fn position_node(
    node_q: Query<
        (&mut SparkNode, &mut Transform, &GridPosition),
        Changed<GridPosition>,
    >,
) {
    for (mut spark_node, mut transform, grid_position) in node_q {
        let new_x = grid_position.x as f32 * (CELL_GAP + CELL_WIDTH);
        let new_y = -(grid_position.y as f32) * (CELL_GAP + CELL_HEIGHT);
        transform.translation.x = new_x;
        transform.translation.y = new_y;
        spark_node.original_translation = transform.translation.clone();
    }
}

fn animate_stimulator_node(
    time: Res<Time>,
    stimulator_q: Query<(
        &mut SyltWorldCanvasScaleFactor,
        &mut StimulatorNode,
        // &mut Transform,
        // &SparkNode,
    )>,
) {
    for (mut scale, mut node /* , mut transform, spark_node */) in stimulator_q
    {
        node.animation_timer.tick(time.delta());
        let v = node.animation_timer.fraction();

        // TODO: center scaling
        let new_scale = bump_logistic(v, 10.).remap(0., 1., 1., 1.2);
        scale.0 = new_scale;

        // transform.translation.x =
        //     spark_node.original_translation.x + 20. * new_scale;
        // transform.translation.y =
        //     spark_node.original_translation.y + 20. * new_scale;
    }
}

fn draw_stimulator_node(
    time: Res<Time>,
    stimulator_q: Query<(&mut SyltWorldScene, &mut StimulatorNode)>,
) {
    for (mut scene, mut node) in stimulator_q {
        let scene = &mut scene.inner;

        scene.reset();

        let pentagon =
            plot_polygon_path((CELL_WIDTH * 0.5, CELL_HEIGHT * 0.5), 40., 5);

        scene.fill(
            vello::peniko::Fill::NonZero,
            Default::default(),
            vello::peniko::Color::new([0.3, 0.6, 0.9, 1.]),
            None,
            &pentagon,
        );

        // if let Some(timer) = node.animation_timer.as_mut() {
        //     timer.tick(time.delta());
        //     if timer.finished() {
        //         node.animation_timer = None;
        //     }
        // }
        //
        // if let Some(timer) = node.animation_timer.as_mut() {
        //     timer.tick(time.delta());
        //     let v = timer.fraction();
        //
        //     let pentagon_enlarged = pentagon.to_path().to_path
        //
        // scene.fill(
        //     vello::peniko::Fill::NonZero,
        //     Default::default(),
        //     vello::peniko::Color::new([0.3, 0.6, 0.9, 1.]),
        //     None,
        //     &pentagon_enlarged,
        // );
        // }
    }
}

fn pulsate_stimulator_node(
    mut cmd: Commands,
    time: Res<Time>,
    stimulator_node_q: Query<(&mut StimulatorNode, &GridPosition)>,
    mut clicker_q: Query<(&GridPosition, &mut ClickerNode)>,
    mut sparks: ResMut<Sparks>,
    sounds: Res<SyltSoundAssets>,
    settings: Res<SyltSettings>,
) {
    let neighbors = [
        Vec2::new(-1., -1.),
        Vec2::new(0., -1.),
        Vec2::new(1., -1.),
        Vec2::new(-1., 0.),
        Vec2::new(1., 0.),
        Vec2::new(-1., 1.),
        Vec2::new(0., 1.),
        Vec2::new(1., 1.),
    ];

    for (mut node, stim_pos) in stimulator_node_q {
        if node.click_timer.tick(time.delta()).finished() {
            for (clicker_pos, mut clicker_node) in clicker_q.iter_mut() {
                let result = Vec2::new(stim_pos.x as f32, stim_pos.y as f32)
                    - Vec2::new(clicker_pos.x as f32, clicker_pos.y as f32);

                if neighbors.contains(&result) {
                    node.animation_timer =
                        Timer::from_seconds(0.25, TimerMode::Once);
                    clicker_node.animation_timer =
                        Timer::from_seconds(0.067, TimerMode::Once);

                    // TODO: delay with a timed event
                    cmd.spawn(play_game_sound_despawn(
                        sounds.pulsate.clone(),
                        &settings,
                    ));
                    cmd.spawn(play_game_sound_despawn(
                        sounds.click.clone(),
                        &settings,
                    ));
                    sparks.0 += 1.;
                }
            }
        }
    }
}

fn draw_clicker_node(
    mut cmd: Commands,
    clicker_q: Query<(Entity, &mut SyltWorldScene, &ClickerNode)>,
) {
    for (entity, mut scene, _cell) in clicker_q {
        let _size = scene.pixel_size;
        let scene = &mut scene.inner;

        scene.reset();

        let shape = vello::kurbo::Circle::new(
            (CELL_WIDTH * 0.5, CELL_HEIGHT * 0.5),
            40.,
        );

        cmd.entity(entity).insert(SyltPickingShape {
            inner: shape.to_path(0.1),
            ..default()
        });

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([0.3, 0.6, 0.9, 1.]),
            None,
            &shape,
        );
    }
}

fn animate_clicker_node(
    time: Res<Time>,
    stimulator_q: Query<(
        &mut SyltWorldCanvasScaleFactor,
        &mut ClickerNode,
        // &mut Transform,
        // &SparkNode,
    )>,
) {
    for (mut scale, mut node /* , mut transform, spark_node */) in stimulator_q
    {
        node.animation_timer.tick(time.delta());
        let v = node.animation_timer.fraction();

        // TODO: center scaling
        let new_scale = bump_logistic(v, 10.).remap(0., 1., 1., 1.2);
        scale.0 = new_scale;

        // transform.translation.x =
        //     spark_node.original_translation.x + 20. * new_scale;
        // transform.translation.y =
        //     spark_node.original_translation.y + 20. * new_scale;
    }
}

fn on_pointer_release(
    trigger: Trigger<Pointer<Released>>,
    mut cmd: Commands,
    clicker_q: Query<&ClickerNode>,
    mut sparks: ResMut<Sparks>,
    sounds: Res<SyltSoundAssets>,
    settings: Res<SyltSettings>,
) {
    if clicker_q.get(trigger.target).is_ok() {
        cmd.spawn(play_game_sound_despawn(sounds.click.clone(), &settings));
        sparks.0 += 1.;
    }
}

fn on_keyboard_press(
    mut cmd: Commands,
    display_shop: Res<ShowBuildMenu>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sparks: ResMut<Sparks>,
    clicker_q: Query<(&GridPosition, &mut ClickerNode)>,
    focused_grid_position: Res<FocusedGridPosition>,
    sounds: Res<SyltSoundAssets>,
    settings: Res<SyltSettings>,
) {
    if display_shop.0 {
        return;
    }

    if keyboard_input.any_just_pressed([
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::KeyI,
    ]) {
        for (grid_position, mut node) in clicker_q {
            if grid_position.x == focused_grid_position.x
                && grid_position.y == focused_grid_position.y
            {
                node.animation_timer =
                    Timer::from_seconds(0.067, TimerMode::Once);
                // Using ButtonInput should prevent holding the key down
                cmd.spawn(play_game_sound_despawn(
                    sounds.click.clone(),
                    &settings,
                ));
                sparks.0 += 1.;
            }
        }
    }
}
