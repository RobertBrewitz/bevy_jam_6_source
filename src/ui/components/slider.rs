use std::ops::RangeInclusive;

use bevy::{
    ecs::system::EntityCommands,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::view::RenderLayers,
};

use crate::{
    canvas::ui_canvas::SyltUiScene,
    ui::{
        cardinal_navigation::{
            SyltCardinalFocusable, SyltCardinalFocusedResource,
        },
        system_set::SyltUiSystem,
    },
    vectors::easings::ease_out_elastic,
};

pub struct SyltSliderPlugin;

impl Plugin for SyltSliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SyltSliderUpdateEvent>();
        app.add_systems(
            Update,
            (render_slider, decrease_slider, increase_slider),
        );
        app.add_systems(
            Update,
            render_slider.in_set(SyltUiSystem::CanvasForeground),
        );
    }
}

#[derive(Component)]
#[require(SyltUiScene, SyltCardinalFocusable, Node, ZIndex)]
pub struct SyltSlider {
    pub range: RangeInclusive<f32>,
    pub slide_animation_timer: Timer,
    pub from: f32,
    pub to: f32,
}

impl Default for SyltSlider {
    fn default() -> Self {
        Self {
            range: 0.0..=1.0,
            slide_animation_timer: Timer::from_seconds(0.4, TimerMode::Once),
            from: 0.0,
            to: 0.0,
        }
    }
}

#[derive(Event)]
pub struct SyltSliderUpdateEvent;

pub trait SyltSliderExt {
    fn spawn_sylt_slider(
        &mut self,
        range: RangeInclusive<f32>,
        default: f32,
        wrapper_bundler: impl Bundle,
        bundle: impl Bundle,
    ) -> EntityCommands;
}

impl SyltSliderExt for Commands<'_, '_> {
    fn spawn_sylt_slider(
        &mut self,
        range: RangeInclusive<f32>,
        value: f32,
        _wrapper_bundler: impl Bundle,
        bundle: impl Bundle,
    ) -> EntityCommands {
        let mut slider = self.spawn((
            Name::new("SyltSlider"),
            RenderLayers::layer(1),
            SyltSlider {
                range,
                from: value,
                to: value,
                ..default()
            },
        ));
        slider.insert(bundle);
        slider
    }
}

// TODO: gardient slider
fn render_slider(
    time: Res<Time>,
    mut left_arrow_q: Query<(
        &ComputedNode,
        &mut SyltUiScene,
        &mut SyltSlider,
        Option<&ComputedNode>,
        Option<&RenderLayers>,
    )>,
) {
    for (node, mut scene, mut slider, _, _) in left_arrow_q.iter_mut() {
        let max = *slider.range.end();
        let min = *slider.range.start();

        let scene = &mut scene.inner;
        scene.reset();

        slider.slide_animation_timer.tick(time.delta());
        let size = node.size();

        let slider_bg =
            vello::kurbo::Rect::new(0., 0., size.x as f64, size.y as f64);

        let slider_value = slider.from.lerp(
            slider.to,
            ease_out_elastic(slider.slide_animation_timer.fraction()),
        );

        let slider_fg = vello::kurbo::Rect::new(
            0.,
            0.,
            (size.x * (slider_value - min) / (max - min)) as f64,
            size.y as f64,
        );

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([0.2, 0.2, 0.2, 1.0]),
            None,
            &slider_bg,
        );

        scene.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::default(),
            vello::peniko::Color::new([0.7, 0.7, 0.7, 1.0]),
            None,
            &slider_fg,
        );
    }
}

fn decrease_slider(
    mut select_q: Query<(Entity, &mut SyltSlider)>,
    current_cardinal_focus: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut key_events: EventReader<KeyboardInput>,
) {
    for event in key_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        if event.key_code == KeyCode::KeyA
            || event.key_code == KeyCode::ArrowLeft
            || event.key_code == KeyCode::KeyH
        {
            let shift = keyboard_input
                .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let ctrl = keyboard_input
                .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

            if let Some(focused_entity) = current_cardinal_focus.0 {
                for (entity, mut slider) in select_q.iter_mut() {
                    if entity != focused_entity {
                        continue;
                    }
                    let max = *slider.range.end();
                    let min_step = max * 0.01;
                    let step = max * 0.05;
                    let max_step = max * 0.20;
                    let step = if ctrl {
                        max * max_step
                    } else if shift {
                        max * min_step
                    } else {
                        max * step
                    };
                    slider.slide_animation_timer.reset();
                    slider.from = slider.to;
                    let new_value =
                        (slider.to - step).max(*slider.range.start());
                    if new_value < min_step {
                        slider.to = *slider.range.start();
                    } else {
                        slider.to = f32::round(new_value * 100.0) / 100.0;
                    }
                }
            }
        }
    }
}

fn increase_slider(
    mut select_q: Query<(Entity, &mut SyltSlider)>,
    current_cardinal_focus: Res<SyltCardinalFocusedResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut key_events: EventReader<KeyboardInput>,
) {
    for event in key_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        if event.key_code == KeyCode::KeyD
            || event.key_code == KeyCode::ArrowRight
            || event.key_code == KeyCode::KeyL
        {
            let shift = keyboard_input
                .any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            let ctrl = keyboard_input
                .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

            if let Some(focused_entity) = current_cardinal_focus.0 {
                for (entity, mut slider) in select_q.iter_mut() {
                    if entity != focused_entity {
                        continue;
                    }

                    let max = *slider.range.end();
                    let min_step = max * 0.01;
                    let step = max * 0.05;
                    let max_step = max * 0.20;
                    let step = if ctrl {
                        max * max_step
                    } else if shift {
                        max * min_step
                    } else {
                        max * step
                    };
                    slider.slide_animation_timer.reset();
                    slider.from = slider.to;
                    let new_value = (slider.to + step).min(*slider.range.end());
                    if max - new_value < min_step {
                        slider.to = *slider.range.end();
                    } else {
                        slider.to = f32::round(new_value * 100.0) / 100.0;
                    }
                }
            }
        }
    }
}
