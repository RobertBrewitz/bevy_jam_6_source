use std::f64::consts::TAU;

pub fn plot_polygon_path(
    center: impl Into<vello::kurbo::Point>,
    radius: f64,
    sides: usize,
) -> vello::kurbo::BezPath {
    let center: vello::kurbo::Point = center.into();
    let mut path = vello::kurbo::BezPath::new();
    let angle_increment = TAU / sides as f64;

    for i in 0..sides as i32 {
        let angle = angle_increment * i as f64;
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        if i == 0 {
            path.move_to((x, y));
        } else {
            path.line_to((x, y));
        }
    }

    path.close_path();
    path
}
