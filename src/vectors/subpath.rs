use vello::kurbo::{BezPath, ParamCurve, ParamCurveArclen, PathSeg, Shape};

/// Extracts a subpath from a path based on a fraction of the total length of the path.
///
/// path: BezPath
/// fraction_length: f64 (0.0 - 1.0)
/// start_length: f64 (>= 0.0)
/// accuracy: f64 (0.0 - 1.0) the lower the more accurate and slower
pub fn extract_subpath_fraction(
    path: &BezPath,
    fraction_length: f64,
    start_length: f64,
    accuracy: f64,
) -> BezPath {
    let total_length = path.segments().map(|s| s.arclen(accuracy)).sum::<f64>();
    let start_length = start_length.rem_euclid(total_length);
    let subpath_length = total_length * fraction_length;

    extract_subpath_absolute(path, subpath_length, start_length, accuracy)
}

/// Extracts a subpath from a path based on an absolute length.
///
/// path: BezPath
/// subpath_length: f64 (>= 0.0)
/// start_length: f64 (>= 0.0)
/// accuracy: f64 (0.0 - 1.0) the lower the more accurate and slower
pub fn extract_subpath_absolute(
    path: &BezPath,
    subpath_length: f64,
    start_length: f64,
    accuracy: f64,
) -> BezPath {
    let total_length = path.segments().map(|s| s.arclen(accuracy)).sum::<f64>();
    let start_length = start_length.rem_euclid(total_length);
    extract_subpath(path, subpath_length, start_length, accuracy)
}

fn extract_subpath(
    path: &BezPath,
    subpath_length: f64,
    start_length: f64,
    accuracy: f64,
) -> BezPath {
    let end_length = start_length + subpath_length;
    let mut sub_path = BezPath::new();

    let segments: Vec<PathSeg> = path.segments().collect();
    let mut path_length_traversed = 0.0;
    let mut subpath_length_remainder = subpath_length;

    loop {
        for segment in &segments {
            if end_length <= path_length_traversed
                || subpath_length_remainder <= 0.0
            {
                break;
            }

            let segment_length = segment.arclen(accuracy);
            let segment_start = path_length_traversed;
            let segment_end = segment_start + segment_length;

            path_length_traversed += segment_length;

            if start_length >= segment_start && start_length <= segment_end {
                let start_t = (start_length - segment_start) / segment_length;
                let end_t = (start_length + subpath_length - segment_start)
                    / segment_length;

                subpath_length_remainder -= segment_length * (1.0 - start_t);
                sub_path.extend(
                    segment
                        .subsegment(start_t..end_t.min(1.0))
                        .to_path(accuracy),
                );
            } else if end_length >= segment_start && end_length <= segment_end {
                let end_t = (start_length + subpath_length - segment_start)
                    / segment_length;

                subpath_length_remainder -= segment_length;
                sub_path.extend(
                    segment.subsegment(0.0..end_t.min(1.0)).to_path(accuracy),
                );
            } else if start_length <= segment_start && end_length >= segment_end
            {
                subpath_length_remainder -= segment_length;
                sub_path.extend(segment.to_path(accuracy));
            }
        }

        if end_length <= path_length_traversed
            || subpath_length_remainder <= 0.0
        {
            break;
        }
    }

    sub_path
}
