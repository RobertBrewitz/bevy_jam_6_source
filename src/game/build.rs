use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};

use crate::{
    canvas::{
        text::{
            SyltFontAxes, SyltText, SyltTextAlign, SyltTextAnchor,
            SyltTextStyle,
        },
        ui_canvas::{
            NoSyltUiScaling, SyltUiScene, SyltUiText, SyltUiUseWorldCoorindates,
        },
        world_canvas::SyltWorldCanvasScaleFactor,
    },
    game::{
        grid::{
            FocusedGridPosition, GridPosition, CELL_GAP, CELL_HEIGHT,
            CELL_WIDTH,
        },
        instructions::InstructionState,
        nodes::{SparkNode, StimulatorNode, STIMULATOR_NODE_COST},
        system_set::{SyltGameSystemSet, SyltPausableSystems},
        Sparks,
    },
    i18n::SyltI18nText,
    menus::SyltMenuState,
    routes::SyltRouterState,
    vectors::polygon::plot_polygon_path,
};

pub struct BuildPlugin;

impl Plugin for BuildPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SyltRouterState::Game), add_shop);

        app.add_systems(OnExit(SyltRouterState::Game), remove_shop);

        app.add_systems(
            Update,
            (on_keyboard_press)
                .in_set(SyltPausableSystems)
                .in_set(SyltGameSystemSet::Input)
                .run_if(in_state(SyltRouterState::Game)),
        );

        app.add_systems(
            Update,
            (spawn_build_menu, despawn_build_menu, position_build_menu)
                .in_set(SyltPausableSystems)
                .in_set(SyltGameSystemSet::Update)
                .run_if(in_state(SyltRouterState::Game)),
        );

        app.add_systems(
            Update,
            (draw_shop, draw_overlay)
                .in_set(SyltPausableSystems)
                .run_if(
                    resource_exists::<ShowBuildMenu>
                        .and(resource_equals(ShowBuildMenu(true))),
                ),
        );
    }
}

fn add_shop(mut cmd: Commands) {
    cmd.insert_resource(ShowBuildMenu(false));
}

fn remove_shop(mut cmd: Commands) {
    cmd.remove_resource::<ShowBuildMenu>();
}

#[derive(Resource, PartialEq)]
pub struct ShowBuildMenu(pub bool);

fn on_keyboard_press(
    mut cmd: Commands,
    mut display_shop: ResMut<ShowBuildMenu>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    spark_node_q: Query<&GridPosition, With<SparkNode>>,
    focused_grid_position: Res<FocusedGridPosition>,
    mut menu_state: ResMut<NextState<SyltMenuState>>,
    mut sparks: ResMut<Sparks>,
) {
    if keyboard_input.any_just_pressed([
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::KeyB,
        KeyCode::KeyI,
    ]) {
        if display_shop.0 {
            if sparks.0 >= STIMULATOR_NODE_COST {
                sparks.0 -= STIMULATOR_NODE_COST;
                let mut animation_timer =
                    Timer::from_seconds(0.25, TimerMode::Once);
                animation_timer.pause();

                cmd.spawn((
                    StateScoped(SyltRouterState::Game),
                    SparkNode::default(),
                    GridPosition::new(
                        focused_grid_position.x,
                        focused_grid_position.y,
                    ),
                    Transform::from_xyz(0., 0., 100.),
                    SyltWorldCanvasScaleFactor(1.0),
                    StimulatorNode {
                        click_timer: Timer::from_seconds(
                            1.,
                            TimerMode::Repeating,
                        ),
                        ..default()
                    },
                ));
                menu_state.set(SyltMenuState::None);
                display_shop.0 = false;
            }
        } else {
            for grid_position in &spark_node_q {
                if grid_position.x == focused_grid_position.x
                    && grid_position.y == focused_grid_position.y
                {
                    return;
                }
            }

            menu_state.set(SyltMenuState::Disabled);
            display_shop.0 = true;
        }
    }

    if display_shop.0 && keyboard_input.just_pressed(KeyCode::Escape) {
        menu_state.set(SyltMenuState::None);
        display_shop.0 = false;
    }
}

const SHOP_OFFSET: f32 = 10.;

#[derive(Component)]
struct BuyMenu;

#[derive(Component)]
struct BuyMenuOverlay;

fn position_build_menu(
    build_menu_q: Query<&mut Transform, With<BuyMenu>>,
    grid_position: Res<FocusedGridPosition>,
) {
    for mut transform in build_menu_q {
        let new_x = grid_position.x as f32 * (CELL_GAP + CELL_WIDTH);
        let new_y = -(grid_position.y as f32) * (CELL_GAP + CELL_HEIGHT);
        transform.translation.x = new_x;
        transform.translation.y = new_y;
    }
}

fn spawn_build_menu(
    mut cmd: Commands,
    show_build_menu: Res<ShowBuildMenu>,
    mut instruction_state: ResMut<NextState<InstructionState>>,
) {
    if show_build_menu.is_changed() && show_build_menu.0 {
        instruction_state.set(InstructionState::Build);

        // spawn overlay
        cmd.spawn((
            RenderLayers::layer(1),
            Pickable::IGNORE,
            StateScoped(SyltRouterState::Game),
            BuyMenuOverlay,
            NoSyltUiScaling,
            SyltUiScene::default(),
        ));

        cmd.spawn((
            RenderLayers::layer(1),
            StateScoped(SyltRouterState::Game),
            BuyMenu,
            NoSyltUiScaling,
            SyltUiUseWorldCoorindates,
            SyltUiScene::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                RenderLayers::layer(1),
                Transform::from_xyz(CELL_WIDTH / 2., -CELL_HEIGHT - 40., 400.),
                SyltUiText,
                SyltText {
                    content: format!("{STIMULATOR_NODE_COST} SP"),
                    ..default()
                },
                SyltTextAnchor::Center,
                SyltTextAlign::Middle,
                SyltTextStyle {
                    font_size: 20.,
                    font_axes: SyltFontAxes {
                        weight: Some(900.),
                        ..default()
                    },
                    brush: vello::peniko::Brush::Solid(
                        vello::peniko::Color::WHITE,
                    ),
                    ..default()
                },
                SyltUiUseWorldCoorindates,
            ));

            parent.spawn((
                RenderLayers::layer(1),
                Transform::from_xyz(CELL_WIDTH / 2., -CELL_HEIGHT - 20., 400.),
                SyltUiText,
                SyltText::default(),
                SyltTextAnchor::Center,
                SyltTextAlign::Middle,
                SyltI18nText::from_key("stimulator node"),
                SyltTextStyle {
                    font_size: 20.,
                    font_axes: SyltFontAxes {
                        weight: Some(900.),
                        ..default()
                    },
                    brush: vello::peniko::Brush::Solid(
                        vello::peniko::Color::WHITE,
                    ),
                    ..default()
                },
                SyltUiUseWorldCoorindates,
            ));
        });
    }
}

fn despawn_build_menu(
    mut cmd: Commands,
    build_menu_q: Query<Entity, With<BuyMenu>>,
    build_menu_overlay_q: Query<Entity, With<BuyMenuOverlay>>,
    show_build_menu: Res<ShowBuildMenu>,
    mut instruction_state: ResMut<NextState<InstructionState>>,
) {
    if show_build_menu.is_changed() && !show_build_menu.0 {
        instruction_state.set(InstructionState::Gameplay);

        for build_menu in &build_menu_q {
            cmd.entity(build_menu).despawn();
        }

        for build_menu_overlay in &build_menu_overlay_q {
            cmd.entity(build_menu_overlay).despawn();
        }
    }
}

fn draw_overlay(
    build_menu_q: Query<&mut SyltUiScene, Added<BuyMenuOverlay>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for mut scene in build_menu_q {
        let scene = &mut scene.inner;

        scene.reset();

        let shape = vello::kurbo::Rect::new(
            0.,
            0.,
            window.physical_width() as f64,
            window.physical_height() as f64,
        );

        scene.fill(
            vello::peniko::Fill::NonZero,
            Default::default(),
            vello::peniko::Color::new([0., 0., 0., 0.7]),
            None,
            &shape,
        );
    }
}

fn draw_shop(build_menu_q: Query<&mut SyltUiScene, With<BuyMenu>>) {
    for mut scene in build_menu_q {
        let scene = &mut scene.inner;

        scene.reset();

        let background = vello::kurbo::Rect::new(
            0.,
            0.,
            10. + CELL_WIDTH as f64,
            10. + CELL_HEIGHT as f64,
        );

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default()
                .with_translation(vello::kurbo::Vec2::new(-5., -5.)),
            vello::peniko::Color::new([1., 1., 1., 0.3]),
            None,
            &background,
        );

        let pentagon =
            plot_polygon_path((CELL_WIDTH * 0.5, CELL_HEIGHT * 0.5), 40., 5);

        scene.fill(
            vello::peniko::Fill::NonZero,
            Default::default(),
            vello::peniko::Color::new([0., 0., 0., 1.]),
            None,
            &pentagon,
        );
    }
}
