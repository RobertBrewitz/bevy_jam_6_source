use crate::{
    canvas::svg::{SyltSvg, SyltSvgGlyph},
    routes::SyltRouterState,
    ui::constants::{SU2, SU4},
};
use bevy::{
    ecs::system::EntityCommands, prelude::*, render::view::RenderLayers,
};

#[derive(Event, Clone, Debug, Reflect)]
pub struct SyltGlyphFocused;

#[derive(Event, Clone, Debug, Reflect)]
pub struct SyltGlyphPressed;

#[derive(Default, Component)]
pub struct SyltGlyph;

pub trait SyltGlyphExt {
    fn spawn_sylt_glyph(
        &mut self,
        glyph: SyltSvgGlyph,
        bundle: impl Bundle,
    ) -> Entity;
}

pub trait SyltGlyphNavigationExt {
    fn navigate_on_click(&mut self, route: SyltRouterState) -> &mut Self;
}

impl SyltGlyphNavigationExt for EntityCommands<'_> {
    fn navigate_on_click(&mut self, route: SyltRouterState) -> &mut Self {
        let route1 = route.clone();
        let route2 = route.clone();

        self.observe(
            move |_: Trigger<SyltGlyphPressed>,
                  mut n: ResMut<NextState<SyltRouterState>>| {
                n.set(route1.clone());
            },
        )
        .observe(
            move |_: Trigger<Pointer<Released>>,
                  mut n: ResMut<NextState<SyltRouterState>>| {
                n.set(route2.clone());
            },
        )
    }
}

impl SyltGlyphExt for Commands<'_, '_> {
    fn spawn_sylt_glyph(
        &mut self,
        glyph: SyltSvgGlyph,
        bundle: impl Bundle,
    ) -> Entity {
        let mut glyph = self.spawn((
            Name::new(format!("{} SyltGlyph", glyph)),
            RenderLayers::layer(1),
            SyltGlyph,
            // SyltUiSvgBrush(vello::peniko::Brush::Gradient(
            //     vello::peniko::Gradient::new_linear(
            //         vello::kurbo::Point::default(),
            //         vello::kurbo::Point::new(64., 0.),
            //     )
            //     .with_stops([
            //         vello::peniko::Color::new([0.8, 0.6, 0.2, 1.0]),
            //         vello::peniko::Color::new([0.8, 0.6, 0.2, 1.0]),
            //     ]),
            // )),
            SyltSvg {
                glyph: glyph.clone(),
                ..default()
            },
            Node {
                padding: UiRect::axes(Val::Px(SU4), Val::Px(SU2)),
                ..default()
            },
        ));
        glyph.insert(bundle);
        glyph.id()
    }
}
