use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};

use crate::canvas::ui_canvas::SyltUiScene;

use super::system_set::SyltUiSystem;

pub struct SyltUiDebugPlugin;

impl Plugin for SyltUiDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_debug_ui).add_systems(
            Update,
            draw_debug_ui_scene.in_set(SyltUiSystem::Layout),
        );
    }
}

#[derive(Component)]
#[require(SyltUiScene)]
pub struct DebugUiScene;

fn draw_debug_ui_scene(
    _windows: Query<&Window, With<PrimaryWindow>>,
    mut scene_q: Query<&mut SyltUiScene, With<DebugUiScene>>,
    ui_element_q: Query<(&ComputedNode, &GlobalTransform)>,
) {
    for mut scene in scene_q.iter_mut() {
        let scene = &mut scene.inner;
        scene.reset();

        for (node, gt) in ui_element_q.iter() {
            let size = node.size();
            let pos = gt.translation().truncate();
            let x = pos.x as f64;
            let y = pos.y as f64;
            let shape =
                &vello::kurbo::Rect::new(0., -3., size.x as f64, size.y as f64);

            // box
            scene.stroke(
                &vello::kurbo::Stroke {
                    width: 1.,
                    ..default()
                },
                vello::kurbo::Affine::translate((x, y)).then_translate(
                    vello::kurbo::Vec2::new(
                        -size.x as f64 / 2.,
                        -size.y as f64 / 2.,
                    ),
                ),
                vello::peniko::Color::new([1., 0., 0., 0.5]),
                None,
                shape,
            );
        }
    }
}

fn setup_debug_ui(mut cmd: Commands) {
    cmd.spawn((
        Name::new("DebugUiScene"),
        RenderLayers::layer(1),
        DebugUiScene,
        ZIndex(1000),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        Pickable::IGNORE,
    ));
}
