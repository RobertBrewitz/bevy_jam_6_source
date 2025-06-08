use std::f64::consts::TAU;

use bevy::prelude::*;
use vello::kurbo::Shape;

use crate::{
    canvas::ui_canvas::SyltUiScene,
    ui::{
        cardinal_navigation::SyltCardinalFocusedResource,
        system_set::SyltUiSystem,
    },
    vectors::{
        gradients::primary_gradient, polygon::plot_polygon_path,
        rectangle::SyltRectExt, subpath::extract_subpath_absolute,
    },
};

use super::button::PIXELS_PER_SECOND;

pub struct FocusAnimationPlugin;

impl Plugin for FocusAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            render_focus_animation.in_set(SyltUiSystem::CanvasBackground),
        );
    }
}

#[derive(Component, Default)]
pub struct SyltFocusPrimary;

fn render_focus_animation(
    mut button_q: Query<
        (Entity, &ComputedNode, &mut SyltUiScene),
        With<SyltFocusPrimary>,
    >,
    time: Res<Time>,
    cardinal_focus: Res<SyltCardinalFocusedResource>,
) {
    for (entity, node, mut scene) in button_q.iter_mut() {
        let size = node.size();
        let scene = &mut scene.inner;
        scene.reset();

        let boundry_shape =
            vello::kurbo::Rect::new(0., 0., size.x as f64, size.y as f64);

        let gradient = primary_gradient(&boundry_shape);

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([0.2, 0.2, 0.2, 1.0]),
            None,
            &boundry_shape,
        );

        if let Some(focused_entity) = cardinal_focus.0 {
            if focused_entity == entity {
                scene.stroke(
                    &vello::kurbo::Stroke {
                        width: 2.0,
                        ..default()
                    },
                    vello::kurbo::Affine::default(),
                    vello::peniko::Color::new([0.8, 0.6, 0.2, 1.0]),
                    None,
                    &boundry_shape.plot_enlarged(2.).into_path(0.1),
                );

                scene.stroke(
                    &vello::kurbo::Stroke {
                        width: 2.0,
                        ..default()
                    },
                    vello::kurbo::Affine::default(),
                    vello::peniko::Color::new([0.9, 0.7, 0.3, 1.0]),
                    None,
                    &extract_subpath_absolute(
                        &boundry_shape.plot_enlarged(2.).into_path(0.1),
                        10.,
                        time.elapsed_secs_f64() * PIXELS_PER_SECOND,
                        0.1,
                    ),
                );

                scene.fill(
                    vello::peniko::Fill::NonZero,
                    vello::kurbo::Affine::default(),
                    &gradient,
                    None,
                    &boundry_shape,
                );

                scene.push_layer(
                    vello::peniko::Mix::Clip,
                    1.0,
                    vello::kurbo::Affine::IDENTITY,
                    &boundry_shape,
                );

                let polygon = plot_polygon_path(
                    (size.x as f64 / 2., size.y as f64 / 2.),
                    size.x as f64 / 2. - 8.,
                    12,
                );

                scene.fill(
                    vello::peniko::Fill::EvenOdd,
                    vello::kurbo::Affine::translate(vello::kurbo::Vec2::new(
                        -size.x as f64 / 2.,
                        -size.y as f64 / 2.,
                    ))
                    .then_rotate(TAU * time.elapsed_secs_f64() * 0.1)
                    .then_translate(
                        vello::kurbo::Vec2::new(
                            size.x as f64 / 2.,
                            size.y as f64 / 2.,
                        ),
                    ),
                    vello::peniko::Color::new([1.0, 1.0, 1.0, 0.2]),
                    None,
                    &polygon,
                );

                scene.pop_layer();
            }
        }
    }
}
