use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use thiserror::Error;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    platform::collections::HashMap,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAsset, RenderAssetPlugin},
    },
    ui::{ContentSize, NodeMeasure},
};

use crate::{cameras::SyltUiCamera, ui::system_set::SyltUiSystem};

pub struct SyltSvgPlugin;

impl Plugin for SyltSvgPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SyltKeyboardGlyphCollection>();

        app.init_asset::<SyltSvgAsset>()
            .init_asset_loader::<SyltSvgLoader>();

        app.add_plugins(ExtractResourcePlugin::<SyltSvgCollection>::default());

        // app.add_systems(Startup, load_svg_collection);

        app.add_systems(
            Update,
            (
                calculate_sylt_svg_content_size_system,
                // set_ui_svg_width_height,
            )
                .in_set(SyltUiSystem::Layout),
        );
    }

    fn finish(&self, app: &mut App) {
        app.add_plugins(RenderAssetPlugin::<SyltSvgAsset>::default());
    }
}

#[derive(Debug, Clone, Component)]
#[require(SyltSvgAnchor)]
pub struct SyltSvg {
    pub glyph: SyltSvgGlyph,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub alpha: f32,
    pub asset_width: f32,
    pub asset_height: f32,
    pub scale_factor: f32,
}

impl Default for SyltSvg {
    fn default() -> Self {
        Self {
            glyph: SyltSvgGlyph::None,
            max_width: None,
            max_height: None,
            alpha: 1.0,
            asset_width: 0.0,
            asset_height: 0.0,
            scale_factor: 1.0,
        }
    }
}

impl SyltSvg {
    pub fn with_max_size(mut self, max_width: f32, max_height: f32) -> Self {
        self.max_width = Some(max_width);
        self.max_height = Some(max_height);
        self.calculate_scale_factor();
        self
    }

    pub fn calculate_scale_factor(&mut self) {
        if self.max_width.is_none() && self.max_height.is_none() {
            self.scale_factor = 1.0;
        }

        if self.asset_width == 0.0 || self.asset_height == 0.0 {
            self.scale_factor = 1.0;
        }

        let max_width = self.max_width.unwrap_or(self.asset_width);
        let max_height = self.max_height.unwrap_or(self.asset_height);
        let size = Vec2::new(self.asset_width, self.asset_height);
        let max_size = Vec2::new(max_width, max_height);
        let fill_scale = max_size / size;

        let res = fill_scale.x.min(fill_scale.y);
        self.scale_factor = res;
    }

    pub fn bb_in_world_space(&self, gtransform: &GlobalTransform) -> Rect {
        let width = self.asset_width * self.scale_factor;
        let height = self.asset_height * self.scale_factor;

        // TODO: transform, look at bevy_vello
        // Convert local coordinates to world coordinates
        let local_min = Vec3::new(-width, -height, 0.0).extend(1.0);
        let local_max = Vec3::new(width, height, 0.0).extend(1.0);

        let min_world = gtransform.compute_matrix() * local_min;
        let max_world = gtransform.compute_matrix() * local_max;

        // Calculate the distance between the vertices to get the size in world space
        let min = Vec2::new(min_world.x, min_world.y);
        let max = Vec2::new(max_world.x, max_world.y);
        Rect { min, max }
    }

    /// Returns the bounding box in screen space
    pub fn bb_in_screen_space(
        &self,
        gtransform: &GlobalTransform,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<Rect> {
        let Rect { min, max } = self.bb_in_world_space(gtransform);
        camera
            .viewport_to_world_2d(camera_transform, min)
            .ok()
            .zip(camera.viewport_to_world_2d(camera_transform, max).ok())
            .map(|(min, max)| Rect { min, max })
    }

    pub fn render(
        &self,
        scene: &mut vello::Scene,
        mut affine: vello::kurbo::Affine,
        svg: &SyltSvgAsset,
        svg_anchor: &SyltSvgAnchor,
    ) {
        let width = svg.width as f64;
        let height = svg.height as f64;

        // TODO: move coloring to main app
        let mut color_scene = vello::Scene::new();

        affine *= vello::kurbo::Affine::translate((0.0, -height));

        match svg_anchor {
            SyltSvgAnchor::TopLeft => {
                affine *= vello::kurbo::Affine::translate((0.0, height));
            }
            SyltSvgAnchor::Left => {
                affine *= vello::kurbo::Affine::translate((0.0, height / 2.0));
            }
            SyltSvgAnchor::BottomLeft => {
                affine *= vello::kurbo::Affine::translate((0.0, 0.0));
            }
            SyltSvgAnchor::Top => {
                affine *=
                    vello::kurbo::Affine::translate((-width / 2.0, height));
            }
            SyltSvgAnchor::Center => {
                affine *= vello::kurbo::Affine::translate((
                    -width / 2.0,
                    height / 2.0,
                ));
            }
            SyltSvgAnchor::Bottom => {
                affine *= vello::kurbo::Affine::translate((-width / 2.0, 0.0));
            }
            SyltSvgAnchor::TopRight => {
                affine *= vello::kurbo::Affine::translate((-width, height));
            }
            SyltSvgAnchor::Right => {
                affine *=
                    vello::kurbo::Affine::translate((-width, height / 2.0));
            }
            SyltSvgAnchor::BottomRight => {
                affine *= vello::kurbo::Affine::translate((-width, 0.0));
            }
        }

        // if let Some(brush) = &extracted_svg.maybe_brush {
        //     color_scene.push_layer(
        //         vello::peniko::BlendMode {
        //             mix: vello::peniko::Mix::Normal,
        //             compose: vello::peniko::Compose::SrcOver,
        //         },
        //         1.0,
        //         Affine::IDENTITY,
        //         &rect,
        //     );
        //
        //     vello_svg::append_tree(&mut color_scene, &svg.tree);
        //
        //     color_scene.push_layer(
        //         vello::peniko::BlendMode {
        //             mix: vello::peniko::Mix::Clip,
        //             compose: vello::peniko::Compose::SrcIn,
        //         },
        //         1.0,
        //         Affine::IDENTITY,
        //         &rect,
        //     );
        //
        //     color_scene.fill(
        //         vello::peniko::Fill::NonZero,
        //         vello::kurbo::Affine::IDENTITY,
        //         &brush.0,
        //         None,
        //         &rect,
        //     );
        //     color_scene.pop_layer();
        //     color_scene.pop_layer();
        //     master_scene.append(&color_scene, Some(affine));
        // } else {
        vello_svg::append_tree(&mut color_scene, &svg.tree);
        scene.append(&color_scene, Some(affine));
        // }
    }
}

fn calculate_sylt_svg_content_size_system(
    mut button_q: Query<
        (&mut ContentSize, &mut SyltSvg, &GlobalTransform),
        Changed<SyltSvg>,
    >,
    camera: Single<
        (&Camera, &GlobalTransform),
        (With<SyltUiCamera>, With<Camera2d>),
    >,
) {
    let (camera, view) = *camera;

    for (mut cs, svg, gtransform) in button_q.iter_mut() {
        if let Some(rect) = svg.bb_in_screen_space(gtransform, camera, view) {
            // TODO: scale from setting
            let size = rect.size();
            let measure = NodeMeasure::Fixed(bevy::ui::FixedMeasure {
                size: Vec2::new(size.x.abs(), size.y.abs()),
            });
            cs.set(measure);
        }
    }
}

// fn set_ui_svg_width_height(
//     sylt_svgs: Res<SyltSvgCollection>,
//     svgs_assets: Res<Assets<SyltSvgAsset>>,
//     mut svgs: Query<&mut SyltSvg, Changed<SyltSvg>>,
// ) {
//     for mut svg in svgs.iter_mut() {
//         let maybe_asset_handle = match svg.glyph {
//             SyltSvgGlyph::None => continue,
//             SyltSvgGlyph::SomeDefault => continue,
//         };
//
//         if let Some(asset_handle) = maybe_asset_handle {
//             if let Some(asset) = svgs_assets.get(asset_handle) {
//                 svg.asset_width = asset.width;
//                 svg.asset_height = asset.height;
//                 svg.calculate_scale_factor();
//             }
//         }
//     }
// }

#[derive(Debug, Clone, Component, Deref, DerefMut)]
pub struct SyltSvgBrush(pub vello::peniko::Brush);

/// Describes how the text is positioned relative to its [`Transform`]. It defaults to [`SyltSvgAnchor::Center`].
#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum SyltSvgAnchor {
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

// fn load_svg_collection(mut cmd: Commands, asset_server: Res<AssetServer>) {
//     let mut generic_glyphs: HashMap<GenericGlyph, Handle<SyltSvgAsset>> =
//         HashMap::default();
//     let mut keyboard_and_mouse_glyphs: HashMap<
//         KeyboardAndMouseGlyph,
//         Handle<SyltSvgAsset>,
//     > = HashMap::default();
//     let mut playstation_glyphs: HashMap<
//         PlayStationGlyph,
//         Handle<SyltSvgAsset>,
//     > = HashMap::default();
//     let mut steam_deck_glyphs: HashMap<SteamDeckGlyph, Handle<SyltSvgAsset>> =
//         HashMap::default();
//     let mut switch_glyphs: HashMap<SwitchGlyph, Handle<SyltSvgAsset>> =
//         HashMap::default();
//     let mut xbox_glyphs: HashMap<XboxGlyph, Handle<SyltSvgAsset>> =
//         HashMap::default();
//
//     for glyph in GenericGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path =
//             format!("Kenney/Input Prompts/Generic/Vector/{}.svg", glyph_str);
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         generic_glyphs.insert(glyph, handle);
//     }
//
//     for glyph in KeyboardAndMouseGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path = format!(
//             "Kenney/Input Prompts/Keyboard & Mouse/Vector/{}.svg",
//             glyph_str
//         );
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         keyboard_and_mouse_glyphs.insert(glyph, handle);
//     }
//
//     for glyph in PlayStationGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path = format!(
//             "Kenney/Input Prompts/PlayStation Series/Vector/{}.svg",
//             glyph_str
//         );
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         playstation_glyphs.insert(glyph, handle);
//     }
//
//     for glyph in SteamDeckGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path =
//             format!("Kenney/Input Prompts/Steam Deck/Vector/{}.svg", glyph_str);
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         steam_deck_glyphs.insert(glyph, handle);
//     }
//
//     for glyph in SwitchGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path = format!(
//             "Kenney/Input Prompts/Nintendo Switch/Vector/{}.svg",
//             glyph_str
//         );
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         switch_glyphs.insert(glyph, handle);
//     }
//
//     for glyph in XboxGlyph::iter() {
//         let glyph_str: Arc<str> = glyph.into();
//         let path = format!(
//             "Kenney/Input Prompts/Xbox Series/Vector/{}.svg",
//             glyph_str
//         );
//         let handle: Handle<SyltSvgAsset> = asset_server.load(path);
//         xbox_glyphs.insert(glyph, handle);
//     }
//
//     cmd.insert_resource(SyltSvgCollection {
//         generic: SyltGenericGlyphCollection(generic_glyphs),
//         keyboard_and_mouse: SyltKeyboardAndMouseGlyphCollection(
//             keyboard_and_mouse_glyphs,
//         ),
//         playstation: SyltPlayStationGlyphCollection(playstation_glyphs),
//         steam_deck: SyltSteamDeckGlyphCollection(steam_deck_glyphs),
//         switch: SyltSwitchGlyphCollection(switch_glyphs),
//         xbox: SyltXboxGlyphCollection(xbox_glyphs),
//     });
// }

#[derive(Debug, Default, Clone)]
pub enum SyltSvgGlyph {
    #[default]
    None,
    SomeDefault,
}

impl Display for SyltSvgGlyph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct SyltSvgCollection {
    // pub generic: SyltGenericGlyphCollection,
    // pub keyboard_and_mouse: SyltKeyboardAndMouseGlyphCollection,
    // pub playstation: SyltPlayStationGlyphCollection,
    // pub steam_deck: SyltSteamDeckGlyphCollection,
    // pub switch: SyltSwitchGlyphCollection,
    // pub xbox: SyltXboxGlyphCollection,
}

impl ExtractResource for SyltSvgCollection {
    type Source = SyltSvgCollection;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

#[derive(Clone, Debug, Asset, TypePath)]
pub struct SyltSvgAsset {
    pub tree: Arc<vello_svg::usvg::Tree>,
    pub width: f32,
    pub height: f32,
    pub alpha: f32,
}

impl RenderAsset for SyltSvgAsset {
    type SourceAsset = SyltSvgAsset;
    type Param = ();
    fn prepare_asset(
        source_asset: Self::SourceAsset,
        _asset_id: AssetId<Self::SourceAsset>,
        _param: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
    ) -> Result<
        Self,
        bevy::render::render_asset::PrepareAssetError<Self::SourceAsset>,
    > {
        Ok(source_asset)
    }
}

#[derive(Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct SyltKeyboardGlyphCollection(
    pub HashMap<Arc<str>, Handle<SyltSvgAsset>>,
);

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SyltSvgLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct SyltSvgLoader;
impl AssetLoader for SyltSvgLoader {
    type Asset = SyltSvgAsset;
    type Settings = ();
    type Error = SyltSvgLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let path = load_context.path().to_owned();
        let ext = path.extension().and_then(std::ffi::OsStr::to_str).ok_or(
            SyltSvgLoaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid file extension",
            )),
        )?;

        match ext {
            "svg" => {
                let maybe_svg_str = std::str::from_utf8(&bytes);
                if maybe_svg_str.is_err() {
                    return Err(SyltSvgLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Could not parse SVG as UTF-8",
                    )));
                }

                let svg_str = maybe_svg_str.unwrap();
                let maybe_tree = vello_svg::usvg::Tree::from_str(
                    svg_str,
                    &vello_svg::usvg::Options::default(),
                );

                if maybe_tree.is_err() {
                    return Err(SyltSvgLoaderError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Could not parse SVG",
                    )));
                }

                let tree = maybe_tree.unwrap();
                let width = tree.size().width();
                let height = tree.size().height();

                Ok(SyltSvgAsset {
                    tree: Arc::new(tree),
                    width,
                    height,
                    alpha: 1.0,
                })
            }
            ext => Err(SyltSvgLoaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid file extension: '{ext}'"),
            ))),
        }
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}
