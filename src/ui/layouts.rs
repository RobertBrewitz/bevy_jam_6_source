use bevy::prelude::*;

use super::constants::{SU4, SU8};

pub fn layout_wrapper() -> impl Bundle {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        display: Display::Flex,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(SU4),
        column_gap: Val::Px(SU4),
        ..default()
    }
}

pub fn flex_col_center_center() -> impl Bundle {
    (Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        max_width: Val::Px(1280.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(SU4),
        column_gap: Val::Px(SU4),
        ..default()
    },)
}

pub fn horizontal_center_layout() -> impl Bundle {
    (Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        row_gap: Val::Px(SU4),
        column_gap: Val::Px(SU4),
        padding: UiRect {
            left: Val::Px(SU8),
            right: Val::Px(SU8),
            top: Val::Px(SU8),
            bottom: Val::Px(SU8),
        },
        ..default()
    },)
}

pub fn label_layout() -> impl Bundle {
    (Node {
        padding: UiRect::axes(Val::Px(SU4), Val::Px(SU4)),
        display: Display::Grid,
        grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
        grid_template_rows: vec![GridTrack::flex(1.0)],
        row_gap: Val::Px(SU4),
        column_gap: Val::Px(SU4),
        ..default()
    },)
}

pub fn label_hint_layout() -> impl Bundle {
    (Node {
        padding: UiRect::axes(Val::Px(SU4), Val::Px(SU4)),
        display: Display::Grid,
        grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
        grid_template_rows: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
        row_gap: Val::Px(SU4),
        column_gap: Val::Px(SU4),
        ..default()
    },)
}
