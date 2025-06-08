use std::{borrow::Cow, ops::Mul, sync::Arc};

use bevy::{
    prelude::*,
    ui::{ContentSize, NodeMeasure},
};
use parley::{
    FontSettings, FontStyle, FontVariation, PositionedLayoutItem,
    RangedBuilder, StyleProperty,
};
use vello::{
    kurbo::Affine,
    peniko::{Brush, Color},
};

use crate::{
    cameras::SyltUiCamera,
    canvas::font_context::{LOCAL_FONT_CONTEXT, LOCAL_LAYOUT_CONTEXT},
    ui::system_set::SyltUiSystem,
};

use super::{
    font_context::get_global_font_context,
    ui_canvas::{SyltUiCanvasScale, SyltUiText},
};

pub struct SyltTextPlugin;

impl Plugin for SyltTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            on_text_content_change.in_set(SyltUiSystem::Layout),
        );

        app.add_systems(
            Update,
            on_ui_scale_changed.run_if(resource_changed::<SyltUiCanvasScale>),
        );
    }
}

#[derive(Debug, Default, Clone, Component)]
#[require(SyltTextAnchor, SyltTextAlign, SyltTextStyle)]
pub struct SyltText {
    pub content: String,
    // TODO: use Val in ui canvas
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Component)]
pub struct SyltTextStyle {
    pub family_name: Arc<str>,
    pub brush: Brush,
    pub font_size: f32,
    /// Line height multiplier.
    pub line_height: f32,
    /// Extra spacing between words.
    pub word_spacing: f32,
    /// Extra spacing between letters.
    pub letter_spacing: f32,
    pub font_axes: SyltFontAxes,
}

impl Default for SyltTextStyle {
    fn default() -> Self {
        Self {
            family_name: "Roboto Mono".into(),
            brush: Brush::Solid(Color::WHITE),
            font_size: 24.0,
            line_height: 1.0,
            word_spacing: 0.0,
            letter_spacing: 0.0,
            font_axes: Default::default(),
        }
    }
}

fn on_ui_scale_changed(
    mut button_q: Query<
        (
            &mut ContentSize,
            &mut SyltText,
            &SyltTextAlign,
            &SyltTextStyle,
            &GlobalTransform,
        ),
        (With<SyltText>, With<SyltUiText>),
    >,
    camera: Single<
        (&Camera, &GlobalTransform),
        (With<SyltUiCamera>, With<Camera2d>),
    >,
    ui_scale: Res<SyltUiCanvasScale>,
) {
    let (camera, view) = *camera;

    for (mut cs, ui_text, text_align, text_style, gtransform) in
        button_q.iter_mut()
    {
        if let Some(rect) = ui_text.bb_in_screen_space(
            gtransform, camera, view, text_style, text_align,
        ) {
            let size = rect.size();
            let width = ui_text.width.unwrap_or(size.x.abs().mul(ui_scale.0));
            let height = ui_text.height.unwrap_or(size.y.abs().mul(ui_scale.0));
            let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure {
                size: Vec2::new(width, height),
            });
            cs.set(measure);
        }
    }
}

fn on_text_content_change(
    mut button_q: Query<
        (
            &mut ContentSize,
            &mut SyltText,
            &SyltTextAlign,
            &SyltTextStyle,
            &GlobalTransform,
        ),
        (Changed<SyltText>, With<SyltUiText>),
    >,
    camera: Single<
        (&Camera, &GlobalTransform),
        (With<SyltUiCamera>, With<Camera2d>),
    >,
    ui_scale: Res<SyltUiCanvasScale>,
) {
    let (camera, view) = *camera;

    for (mut cs, ui_text, text_align, text_style, gtransform) in
        button_q.iter_mut()
    {
        if let Some(rect) = ui_text.bb_in_screen_space(
            gtransform, camera, view, text_style, text_align,
        ) {
            let size = rect.size();
            let width = ui_text.width.unwrap_or(size.x.abs().mul(ui_scale.0));
            let height = ui_text.height.unwrap_or(size.y.abs().mul(ui_scale.0));
            let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure {
                size: Vec2::new(width, height),
            });
            cs.set(measure);
        }
    }
}

/// Describes the variable axes of a font.
///
/// https://fonts.google.com/knowledge/introducing_type/introducing_variable_fonts
///
/// Each axis is optional and only present if the font supports it.
#[derive(Debug, Default, Clone)]
pub struct SyltFontAxes {
    /// wght variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/weight_axis
    pub weight: Option<f32>,
    /// wdth variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/width_axis
    pub width: Option<f32>,
    /// opsz variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/optical_size_axis
    pub optical_size: Option<f32>,
    /// ital variable axis only if the font supports it
    ///
    /// Mutually exclusive with `slant`.
    ///
    /// https://fonts.google.com/knowledge/glossary/italic_axis
    pub italic: bool,
    /// slnt variable axis only if the font supports it
    ///
    /// Mutually exclusive with `italic`.
    ///
    /// If italic is true, slant will be ignored.
    ///
    /// https://fonts.google.com/knowledge/glossary/slant_axis
    pub slant: Option<f32>,

    /// GRAD variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/grade_axis
    pub grade: Option<f32>,

    /// XOPQ variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/xopq_axis
    pub thick_stroke: Option<f32>,
    /// yopq variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/yopq_axis
    pub thin_stroke: Option<f32>,

    /// XTRA variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/xtra_axis
    pub counter_width: Option<f32>,

    /// YTUC variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytuc_axis
    pub uppercase_height: Option<f32>,
    /// YTLC variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytlc_axis
    pub lowercase_height: Option<f32>,

    /// YTAS variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytas_axis
    pub ascender_height: Option<f32>,
    /// YTDE variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytde_axis
    pub descender_depth: Option<f32>,

    /// YTFI variable axis only if the font supports it
    ///
    /// https://fonts.google.com/knowledge/glossary/ytfi_axis
    pub figure_height: Option<f32>,
}

/// Describes how the text is positioned relative to its [`Transform`]. It defaults to [`SyltTextAnchor::Center`].
#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum SyltTextAnchor {
    /// Bounds start from the render position and advance up and to the right.
    BottomLeft,
    /// Bounds start from the render position and advance up.
    Bottom,
    /// Bounds start from the render position and advance up and to the left.
    BottomRight,

    /// Bounds start from the render position and advance right.
    Left,
    /// Bounds start from the render position and advance equally on both axes.
    #[default]
    Center,
    /// Bounds start from the render position and advance left.
    Right,

    /// Bounds start from the render position and advance down and to the right.
    TopLeft,
    /// Bounds start from the render position and advance down.
    Top,
    /// Bounds start from the render position and advance down and to the left.
    TopRight,
}

#[derive(Default, Component, Clone, Copy, PartialEq, Eq)]
pub enum SyltTextAlign {
    #[default]
    Start,
    End,
    Left,
    Middle,
    Right,
    Justified,
}

impl From<&SyltTextAlign> for parley::Alignment {
    fn from(value: &SyltTextAlign) -> Self {
        match value {
            SyltTextAlign::Start => parley::Alignment::Start,
            SyltTextAlign::End => parley::Alignment::End,
            SyltTextAlign::Left => parley::Alignment::Left,
            SyltTextAlign::Middle => parley::Alignment::Middle,
            SyltTextAlign::Right => parley::Alignment::Right,
            SyltTextAlign::Justified => parley::Alignment::Justified,
        }
    }
}

impl SyltText {
    pub fn sizeof(
        &self,
        text_style: &SyltTextStyle,
        text_align: &SyltTextAlign,
    ) -> Vec2 {
        LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
            if font_context.is_none() {
                *font_context = Some(get_global_font_context().clone());
            }

            let font_context = font_context.as_mut().unwrap();

            LOCAL_LAYOUT_CONTEXT.with_borrow_mut(|layout_context| {
                let mut builder = layout_context.ranged_builder(
                    font_context,
                    &self.content,
                    1.0,
                    // TODO: this should maybe be false, added in parley 0.4.0
                    true,
                );

                self.apply_font_styles(&mut builder, text_style);
                self.apply_variable_axes(&mut builder, text_style);

                builder.push_default(StyleProperty::FontStack(
                    parley::FontStack::Single(parley::FontFamily::Named(
                        Cow::Borrowed(&text_style.family_name),
                    )),
                ));

                let mut layout = builder.build(&self.content);
                let max_advance = self.width;
                layout.break_all_lines(max_advance);
                layout.align(
                    max_advance,
                    text_align.into(),
                    parley::AlignmentOptions::default(),
                );

                let width = if self.width.is_some() {
                    self.width.unwrap()
                } else {
                    layout.width()
                };

                let height = if self.height.is_some() {
                    self.height.unwrap()
                } else {
                    layout.height()
                };

                Vec2::new(width, height)
            })
        })
    }

    /// Returns the bounding box in world space
    pub fn bb_in_world_space(
        &self,
        transform: &GlobalTransform,
        text_style: &SyltTextStyle,
        text_align: &SyltTextAlign,
    ) -> Rect {
        let size = self.sizeof(text_style, text_align);

        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(0.0, 0.0, 0.0).extend(1.0);
        let local_max = Vec3::new(size.x, size.y, 0.0).extend(1.0);
        let min_world = transform.compute_matrix() * local_min;
        let max_world = transform.compute_matrix() * local_max;

        // Calculate the distance between the vertices to get the size in world space
        let min = Vec2::new(min_world.x, min_world.y);
        let max = Vec2::new(max_world.x, max_world.y);

        Rect { min, max }
    }

    /// Returns the bounding box in screen space
    pub fn bb_in_screen_space(
        &self,
        transform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
        text_style: &SyltTextStyle,
        text_align: &SyltTextAlign,
    ) -> Option<Rect> {
        let Rect { min, max } =
            self.bb_in_world_space(transform, text_style, text_align);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .ok()
            .zip(camera.viewport_to_world_2d(camera_transform, max).ok())
            .map(|(min, max)| Rect { min, max })
    }

    /// Applies the font styles to the text
    ///
    /// font_size - font size
    /// line_height - line height
    /// word_spacing - extra spacing between words
    /// letter_spacing - extra spacing between letters
    fn apply_font_styles(
        &self,
        builder: &mut RangedBuilder<'_, Brush>,
        text_style: &SyltTextStyle,
    ) {
        builder.push_default(StyleProperty::FontSize(text_style.font_size));
        builder.push_default(StyleProperty::LineHeight(text_style.line_height));
        builder
            .push_default(StyleProperty::WordSpacing(text_style.word_spacing));
        builder.push_default(StyleProperty::LetterSpacing(
            text_style.letter_spacing,
        ));
    }

    /// Applies the variable axes to the text
    ///
    /// wght - font weight
    /// wdth - font width
    /// opsz - optical size
    /// ital - italic
    /// slnt - slant
    /// GRAD - grade
    /// XOPQ - thick stroke
    /// YOPQ - thin stroke
    /// YTUC - uppercase height
    /// YTLC - lowercase height
    /// YTAS - ascender height
    /// YTDE - descender depth
    /// YTFI - figure height
    fn apply_variable_axes(
        &self,
        builder: &mut RangedBuilder<'_, Brush>,
        text_style: &SyltTextStyle,
    ) {
        let axes = &text_style.font_axes;
        let mut variable_axes: Vec<FontVariation> = vec![];

        if let Some(weight) = axes.weight {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("wght"),
                value: weight,
            });
        }

        if let Some(width) = axes.width {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("wdth"),
                value: width,
            });
        }

        if let Some(optical_size) = axes.optical_size {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("opsz"),
                value: optical_size,
            });
        }

        if let Some(grade) = axes.grade {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("GRAD"),
                value: grade,
            });
        }

        if let Some(thick_stroke) = axes.thick_stroke {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("XOPQ"),
                value: thick_stroke,
            });
        }

        if let Some(thin_stroke) = axes.thin_stroke {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YOPQ"),
                value: thin_stroke,
            });
        }

        if let Some(counter_width) = axes.counter_width {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("XTRA"),
                value: counter_width,
            });
        }

        if let Some(uppercase_height) = axes.uppercase_height {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YTUC"),
                value: uppercase_height,
            });
        }

        if let Some(lowercase_height) = axes.lowercase_height {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YTLC"),
                value: lowercase_height,
            });
        }

        if let Some(ascender_height) = axes.ascender_height {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YTAS"),
                value: ascender_height,
            });
        }

        if let Some(descender_depth) = axes.descender_depth {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YTDE"),
                value: descender_depth,
            });
        }

        if let Some(figure_height) = axes.figure_height {
            variable_axes.push(parley::swash::Setting {
                tag: parley::swash::tag_from_str_lossy("YTFI"),
                value: figure_height,
            });
        }

        if axes.italic {
            builder.push_default(StyleProperty::FontStyle(FontStyle::Italic));
        } else if axes.slant.is_some() {
            builder.push_default(StyleProperty::FontStyle(FontStyle::Oblique(
                axes.slant,
            )));
        }

        builder.push_default(StyleProperty::FontVariations(
            FontSettings::List(variable_axes.into()),
        ));
    }

    pub fn render(
        &self,
        scene: &mut vello::Scene,
        mut affine: vello::kurbo::Affine,
        text_style: &SyltTextStyle,
        text_anchor: &SyltTextAnchor,
        text_align: &SyltTextAlign,
    ) {
        LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
            if font_context.is_none() {
                *font_context = Some(get_global_font_context().clone());
            }

            let font_context = font_context.as_mut().unwrap();

            LOCAL_LAYOUT_CONTEXT.with_borrow_mut(|layout_context| {
                let mut builder = layout_context.ranged_builder(
                    font_context,
                    &self.content,
                    1.0,
                    // TODO: this should maybe be false, added in parley 0.4.0
                    true,
                );

                self.apply_font_styles(&mut builder, text_style);
                self.apply_variable_axes(&mut builder, text_style);

                builder.push_default(StyleProperty::FontStack(
                    parley::FontStack::Single(parley::FontFamily::Named(
                        Cow::Borrowed(&text_style.family_name),
                    )),
                ));

                let mut layout = builder.build(&self.content);
                let max_advance = self.width;

                layout.break_all_lines(max_advance);
                layout.align(
                    max_advance,
                    text_align.into(),
                    parley::AlignmentOptions::default(),
                );

                let width = if self.width.is_some() {
                    self.width.unwrap()
                } else {
                    layout.width()
                } as f64;

                let height = if self.height.is_some() {
                    self.height.unwrap()
                } else {
                    layout.height()
                } as f64;

                // NOTE: Parley aligns differently than our previous skrifa implementation
                //      so we need to adjust the transform to match the previous behavior
                affine *= vello::kurbo::Affine::translate((0.0, -height));

                match text_anchor {
                    SyltTextAnchor::TopLeft => {
                        affine *=
                            vello::kurbo::Affine::translate((0.0, height));
                    }
                    SyltTextAnchor::Left => {
                        affine *= vello::kurbo::Affine::translate((
                            0.0,
                            height / 2.0,
                        ));
                    }
                    SyltTextAnchor::BottomLeft => {
                        affine *= vello::kurbo::Affine::translate((0.0, 0.0));
                    }
                    SyltTextAnchor::Top => {
                        affine *= vello::kurbo::Affine::translate((
                            -width / 2.0,
                            height,
                        ));
                    }
                    SyltTextAnchor::Center => {
                        affine *= vello::kurbo::Affine::translate((
                            -width / 2.0,
                            height / 2.0,
                        ));
                    }
                    SyltTextAnchor::Bottom => {
                        affine *= vello::kurbo::Affine::translate((
                            -width / 2.0,
                            0.0,
                        ));
                    }
                    SyltTextAnchor::TopRight => {
                        affine *=
                            vello::kurbo::Affine::translate((-width, height));
                    }
                    SyltTextAnchor::Right => {
                        affine *= vello::kurbo::Affine::translate((
                            -width,
                            height / 2.0,
                        ));
                    }
                    SyltTextAnchor::BottomRight => {
                        affine *=
                            vello::kurbo::Affine::translate((-width, 0.0));
                    }
                }

                for line in layout.lines() {
                    for item in line.items() {
                        let PositionedLayoutItem::GlyphRun(glyph_run) = item
                        else {
                            continue;
                        };

                        let mut x = glyph_run.offset();
                        let y = glyph_run.baseline();
                        let run = glyph_run.run();
                        let font = run.font();
                        let font_size = run.font_size();
                        let synthesis = run.synthesis();
                        let glyph_xform = synthesis.skew().map(|angle| {
                            Affine::skew(angle.to_radians().tan() as f64, 0.0)
                        });

                        scene
                            .draw_glyphs(font)
                            .brush(&text_style.brush)
                            .hint(true)
                            .transform(affine)
                            .glyph_transform(glyph_xform)
                            .font_size(font_size)
                            .normalized_coords(run.normalized_coords())
                            .draw(
                                vello::peniko::Fill::NonZero,
                                glyph_run.glyphs().map(|glyph| {
                                    let gx = x + glyph.x;
                                    let gy = y - glyph.y;
                                    x += glyph.advance;
                                    vello::Glyph {
                                        id: glyph.id as _,
                                        x: gx,
                                        y: gy,
                                    }
                                }),
                            );
                    }
                }
            });
        })
    }
}
