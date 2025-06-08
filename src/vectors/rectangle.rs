use crate::ui::constants::SU2;

pub trait SyltRectExt {
    fn plot_rect_corners(
        &self,
        rad: f64,
        padding: f64,
    ) -> vello::kurbo::BezPath;
    fn plot_enlarged(&self, grow: f64) -> vello::kurbo::BezPath;
    fn plot_rect_glare(&self) -> vello::kurbo::BezPath;
}

impl SyltRectExt for vello::kurbo::Rect {
    fn plot_rect_corners(
        &self,
        rad: f64,
        padding: f64,
    ) -> vello::kurbo::BezPath {
        let mut path = vello::kurbo::BezPath::new();
        let origin = self.origin();

        let top_left = origin + vello::kurbo::Vec2::new(-padding, -padding);

        let top_right = origin
            + vello::kurbo::Vec2::new(self.width(), 0.)
            + vello::kurbo::Vec2::new(padding, -padding);

        let bottom_right = origin
            + vello::kurbo::Vec2::new(self.width(), self.height())
            + vello::kurbo::Vec2::new(padding, padding);

        let bottom_left = origin
            + vello::kurbo::Vec2::new(0., self.height())
            + vello::kurbo::Vec2::new(-padding, padding);

        path.move_to(top_left + vello::kurbo::Vec2::new(0., rad));
        path.line_to(top_left);
        path.line_to(top_left + vello::kurbo::Vec2::new(rad, 0.));

        path.move_to(top_right + vello::kurbo::Vec2::new(-rad, 0.));
        path.line_to(top_right);
        path.line_to(top_right + vello::kurbo::Vec2::new(0., rad));

        path.move_to(bottom_right + vello::kurbo::Vec2::new(0., -rad));
        path.line_to(bottom_right);
        path.line_to(bottom_right + vello::kurbo::Vec2::new(-rad, 0.));

        path.move_to(bottom_left + vello::kurbo::Vec2::new(rad, 0.));
        path.line_to(bottom_left);
        path.line_to(bottom_left + vello::kurbo::Vec2::new(0., -rad));

        path
    }

    fn plot_enlarged(&self, grow: f64) -> vello::kurbo::BezPath {
        let mut path = vello::kurbo::BezPath::new();
        let origin = self.origin();

        let top_left = origin + vello::kurbo::Vec2::new(-grow, -grow);

        let top_right = origin
            + vello::kurbo::Vec2::new(self.width(), 0.)
            + vello::kurbo::Vec2::new(grow, -grow);

        let bottom_right = origin
            + vello::kurbo::Vec2::new(self.width(), self.height())
            + vello::kurbo::Vec2::new(grow, grow);

        let bottom_left = origin
            + vello::kurbo::Vec2::new(0., self.height())
            + vello::kurbo::Vec2::new(-grow, grow);

        path.move_to(top_left);
        path.line_to(top_right);
        path.line_to(bottom_right);
        path.line_to(bottom_left);
        path.close_path();

        path
    }

    fn plot_rect_glare(&self) -> vello::kurbo::BezPath {
        let mut path = vello::kurbo::BezPath::new();
        let origin = self.origin();

        let start = origin
            + vello::kurbo::Vec2::new(self.width(), SU2 as f64)
            + vello::kurbo::Vec2::new(-self.width() / 3., 0.);

        path.move_to(start);
        path.line_to(
            origin
                + vello::kurbo::Vec2::new(self.width(), SU2 as f64)
                + vello::kurbo::Vec2::new(-SU2 as f64, 0.),
        );

        path
    }
}
