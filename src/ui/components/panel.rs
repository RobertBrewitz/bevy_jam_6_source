use bevy::prelude::*;

use crate::canvas::ui_canvas::SyltUiScene;

pub struct SyltUiPanelPlugin;

impl Plugin for SyltUiPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_sylt_ui_panel_system);
    }
}

#[derive(Component)]
pub struct SyltPanel;

fn render_sylt_ui_panel_system(
    mut query: Query<(&ComputedNode, &mut SyltUiScene), With<SyltPanel>>,
) {
    for (node, mut scene) in query.iter_mut() {
        let scene = &mut scene.inner;
        scene.reset();
        let size = node.size();
        let shape = vello::kurbo::RoundedRect::new(
            0.,
            0.,
            size.x as f64,
            size.y as f64,
            4.,
        );
        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([0., 0., 0., 0.25]),
            None,
            &shape,
        );
        scene.stroke(
            &vello::kurbo::Stroke {
                width: 1.,
                ..default()
            },
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([1., 0., 0., 0.5]),
            None,
            &shape,
        );
    }
}
