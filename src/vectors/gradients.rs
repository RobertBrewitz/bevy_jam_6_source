pub fn primary_gradient(rect: &vello::kurbo::Rect) -> vello::peniko::Gradient {
    vello::peniko::Gradient::new_linear(
        vello::kurbo::Point::default(),
        vello::kurbo::Point::new(rect.width(), 0.),
    )
    .with_stops([
        vello::peniko::Color::new([0.8, 0.6, 0.2, 0.9]),
        vello::peniko::Color::new([0.8, 0.6, 0.2, 0.5]),
    ])
}
