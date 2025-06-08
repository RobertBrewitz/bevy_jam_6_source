use std::cmp::Reverse;

use bevy::{
    asset::{load_internal_asset, weak_handle, RenderAssetUsages},
    math::FloatOrd,
    picking::{
        backend::{HitData, PointerHits},
        pointer::PointerLocation,
        PickSet,
    },
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::{Indices, MeshVertexBufferLayoutRef, VertexBufferLayout},
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::TemporaryRenderEntity,
        texture::GpuImage,
        view::{ExtractedView, NoFrustumCulling, RenderLayers},
        Extract, Render, RenderApp, RenderSet,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
    window::{PrimaryWindow, WindowResized},
};
use vello::{
    kurbo::Shape,
    wgpu::{
        Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension,
        TextureFormat, TextureUsages, VertexFormat, VertexStepMode,
    },
    RenderParams,
};

use crate::cameras::SyltWorldCamera;

use super::{
    svg::{
        SyltSvg, SyltSvgAnchor, SyltSvgAsset, SyltSvgCollection, SyltSvgGlyph,
    },
    text::{SyltText, SyltTextAlign, SyltTextAnchor, SyltTextStyle},
    VelloRenderer,
};

pub struct SyltWorldCanvasPlugin;

impl Plugin for SyltWorldCanvasPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            WORLD_SHADER_HANDLE,
            "shaders/screen_space_render_target.wgsl",
            Shader::from_wgsl
        );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            ExtractSchedule,
            (extract_sylt_text, extract_sylt_scene, extract_sylt_svg),
        );
        render_app.add_systems(
            Render,
            (
                prepare_scene_affines,
                prepare_text_affines,
                prepare_svg_affines,
            )
                .in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            (render_world_space_canvas)
                .chain()
                .in_set(RenderSet::Render)
                .run_if(resource_exists::<RenderDevice>),
        );

        app.register_type::<WorldSpaceCanvasTexture>();
        app.register_type::<WorldSpaceCanvas>();

        app.insert_resource(SyltWorldCanvasScale::default());
        app.add_plugins(
            ExtractResourcePlugin::<SyltWorldCanvasScale>::default(),
        );

        app.add_plugins(
            Material2dPlugin::<WorldSpaceCanvasMaterial>::default(),
        );

        // Screen Canvas (for rendering)
        app.add_plugins(
            ExtractResourcePlugin::<WorldSpaceCanvasTexture>::default(),
        );
        app.add_systems(Startup, setup_world_space_canvas_mesh);
        app.add_systems(Update, resize_world_space_canvas);
        app.add_systems(PreUpdate, sylt_scene_picking.in_set(PickSet::Backend));
    }
}

#[derive(Default, Clone, Component, Deref)]
pub struct SyltWorldCanvasScaleFactor(pub f32);

/// Affine y-down applied with affine
#[derive(Default, Clone, Component)]
pub struct SyltPickingShape {
    pub inner: vello::kurbo::BezPath,
    pub affine: vello::kurbo::Affine,
}

#[derive(Resource, Clone)]
pub struct SyltWorldCanvasScale(pub f32);

impl Default for SyltWorldCanvasScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ExtractResource for SyltWorldCanvasScale {
    type Source = SyltWorldCanvasScale;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

/// Transform y-up
/// Drawing with y-down
#[derive(Default, Clone, Component)]
#[require(Transform)]
pub struct SyltWorldScene {
    pub inner: vello::Scene,
    pub pixel_size: Vec2,
}

impl SyltWorldScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for SyltWorldScene {
    fn from(scene: vello::Scene) -> Self {
        Self {
            inner: scene,
            ..default()
        }
    }
}

/// Transform y-up
#[derive(Default, Clone, Component)]
#[require(Transform)]
pub struct SyltWorldText;

/// Transform y-up
#[derive(Default, Clone, Component)]
#[require(Transform)]
pub struct SyltWorldSvg;

const WORLD_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("97f49b2f-33b4-4b44-9e18-23a1c704f098");

#[derive(Reflect, Component)]
#[reflect(Component)]
struct WorldSpaceCanvas(pub Handle<WorldSpaceCanvasMaterial>);

// TODO: Probably have to extract manually to do it on every frame
#[derive(Reflect, Resource, Default, Clone)]
#[reflect(Resource)]
struct WorldSpaceCanvasTexture(pub Handle<Image>);

impl ExtractResource for WorldSpaceCanvasTexture {
    type Source = WorldSpaceCanvasTexture;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

#[derive(AsBindGroup, TypePath, Asset, Clone)]
struct WorldSpaceCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for WorldSpaceCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        WORLD_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        WORLD_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(target) = descriptor.fragment.as_mut() {
            let mut_targets = &mut target.targets;
            if let Some(Some(target)) = mut_targets.get_mut(0) {
                target.blend = Some(vello::wgpu::BlendState::ALPHA_BLENDING);
            }
        }

        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            VertexFormat::Float32x2,
        ];

        let vertex_layout = VertexBufferLayout::from_vertex_formats(
            VertexStepMode::Vertex,
            formats,
        );

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

fn setup_world_space_canvas_mesh(
    mut cmd: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut canvas_materials: ResMut<Assets<WorldSpaceCanvasMaterial>>,
    mut render_target_mesh_handle: Local<Option<Handle<Mesh>>>,
) {
    // 2D Mesh
    let mesh_handle = render_target_mesh_handle.get_or_insert_with(|| {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        let verts = vec![
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);

        let uv = vec![[0., 0.], [1.0, 0.], [1.0, 1.0], [0., 1.0]];
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);

        let indices = vec![0, 1, 2, 0, 2, 3];
        mesh.insert_indices(Indices::U32(indices));

        meshes.add(mesh)
    });

    let image_size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        depth_or_array_layers: 1,
    };

    // texture image
    let mut texture_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("SyltWorldCanvasTexture"),
            size: image_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    texture_image.resize(image_size);
    let image_handle = images.add(texture_image);
    let world_space_canvas_texture =
        WorldSpaceCanvasTexture(image_handle.clone());
    cmd.insert_resource(world_space_canvas_texture);

    // 2D Mesh Material
    let material_handle = canvas_materials.add(WorldSpaceCanvasMaterial {
        texture: image_handle,
    });

    cmd.spawn((
        Name::new("SyltWorldCanvas"),
        WorldSpaceCanvas(material_handle.clone()),
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(material_handle),
        NoFrustumCulling,
        RenderLayers::layer(0),
    ));
}

fn resize_world_space_canvas(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    mut window_resized_events: EventReader<WindowResized>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut canvas: Query<&mut WorldSpaceCanvas>,
    mut canvas_texture: ResMut<WorldSpaceCanvasTexture>,
    mut canvas_material_assets: ResMut<Assets<WorldSpaceCanvasMaterial>>,
) {
    if window_resized_events.read().last().is_some() {
        let image_size = Extent3d {
            width: window.physical_width(),
            height: window.physical_height(),
            depth_or_array_layers: 1,
        };
        if image_size.width == 0 || image_size.height == 0 {
            return;
        }

        // TODO: Asset should probably be a resource
        for material in canvas.iter_mut() {
            let mut texture_image = Image {
                texture_descriptor: TextureDescriptor {
                    label: Some("WorldSpaceCanvasTexture"),
                    size: image_size,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8Unorm,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::STORAGE_BINDING,
                    view_formats: &[],
                },
                ..default()
            };
            texture_image.resize(image_size);
            let texture_image_handle = images.add(texture_image);
            let world_space_canvas_texture =
                WorldSpaceCanvasTexture(texture_image_handle.clone());
            cmd.insert_resource(world_space_canvas_texture.clone());

            canvas_texture.0 = texture_image_handle.clone();
            canvas_material_assets.get_mut(&material.0).unwrap().texture =
                texture_image_handle;
        }
    }
}

#[derive(Component)]
struct ExtractedSyltScene {
    pub inner: SyltWorldScene,
    pub transform: GlobalTransform,
    pub maybe_scale_factor: Option<SyltWorldCanvasScaleFactor>,
}

#[derive(Debug, Component)]
struct PreparedSceneAffine(pub vello::kurbo::Affine);

fn extract_sylt_scene(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        With<SyltWorldCamera>,
    >,

    scenes: Extract<
        Query<(
            &GlobalTransform,
            &SyltWorldScene,
            Option<&RenderLayers>,
            Option<&SyltWorldCanvasScaleFactor>,
        )>,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (transform, scene, maybe_render_layers, maybe_scale_factor) in
        scenes.iter()
    {
        // Check if any camera renders this asset
        if views.iter().any(|(_camera, camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltScene {
                    transform: *transform,
                    inner: scene.clone(),
                    maybe_scale_factor: maybe_scale_factor.cloned(),
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_scene_affines(
    mut cmd: Commands,
    views: Single<(&ExtractedCamera, &ExtractedView), With<SyltWorldCamera>>,
    scenes: Query<(Entity, &ExtractedSyltScene)>,
    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (camera, view) = *views;
    let viewport_dimension_in_pixels = camera.physical_viewport_size.unwrap();
    let (center_x, center_y) = (
        (viewport_dimension_in_pixels.x as f32 * 0.5),
        (viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    // 0,0 is the center of the 2d viewport in Bevy
    // so we need to transpose the matrix to have Vello
    // use the center of the viewport as 0,0 as well
    let screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [center_x, 0.0, 0.0, center_x / world_scale.0], // x
        [0.0, center_y, 0.0, center_y / world_scale.0], // y
        [0.0, 0.0, 1.0, 0.0],                           // z
        [0.0, 0.0, 0.0, 1.0],                           // w
    ])
    .transpose();

    for (
        entity,
        ExtractedSyltScene {
            transform,
            maybe_scale_factor,
            ..
        },
    ) in scenes.iter()
    {
        let mut model_matrix =
            transform.compute_matrix().mul_scalar(world_scale.0);
        let mut view_matrix = view.world_from_view.compute_matrix();

        // flip the y-axis as Vello uses y-down and Bevy uses y-up
        model_matrix.w_axis.y *= -1.0;
        view_matrix.w_axis.y *= -1.0;

        if let Some(scale_factor) = maybe_scale_factor {
            model_matrix.x_axis.x *= **scale_factor;
            model_matrix.y_axis.y *= **scale_factor;
        }

        let projection_matrix = view.clip_from_view;
        let view_projection_matrix = projection_matrix * view_matrix.inverse();

        let model_render_transform =
            screen_matrix * view_projection_matrix * model_matrix;
        let transform = model_render_transform.to_cols_array();
        let transform: [f64; 6] = [
            transform[0] as f64,
            -transform[1] as f64,
            -transform[4] as f64,
            transform[5] as f64,
            transform[12] as f64,
            transform[13] as f64,
        ];
        cmd.entity(entity)
            .insert(PreparedSceneAffine(vello::kurbo::Affine::new(transform)));
    }
}

#[derive(Component)]
struct ExtractedSyltWorldText {
    pub inner: SyltText,
    pub text_style: SyltTextStyle,
    pub text_align: SyltTextAlign,
    pub text_anchor: SyltTextAnchor,
    pub transform: GlobalTransform,
}

#[derive(Debug, Component)]
struct PreparedTextAffine(pub vello::kurbo::Affine);

fn extract_sylt_text(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        With<SyltWorldCamera>,
    >,

    texts: Extract<
        Query<
            (
                &GlobalTransform,
                &SyltText,
                &SyltTextStyle,
                &SyltTextAnchor,
                &SyltTextAlign,
                Option<&RenderLayers>,
            ),
            With<SyltWorldText>,
        >,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        transform,
        text,
        text_style,
        text_anchor,
        text_align,
        maybe_render_layers,
    ) in texts.iter()
    {
        // Check if any camera renders this asset
        if views.iter().any(|(_camera, maybe_camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(maybe_camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltWorldText {
                    inner: text.clone(),
                    text_style: text_style.clone(),
                    text_align: *text_align,
                    text_anchor: *text_anchor,
                    transform: *transform,
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_text_affines(
    mut cmd: Commands,
    views: Single<(&ExtractedCamera, &ExtractedView), With<SyltWorldCamera>>,
    texts: Query<(Entity, &ExtractedSyltWorldText)>,
    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (camera, view) = *views;
    let viewport_dimension_in_pixels = camera.physical_viewport_size.unwrap();
    let (center_x, center_y) = (
        (viewport_dimension_in_pixels.x as f32 * 0.5),
        (viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    // 0,0 is the center of the 2d viewport in Bevy
    // so we need to transpose the matrix to have Vello
    // use the center of the viewport as 0,0 as well
    let screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [center_x, 0.0, 0.0, center_x / world_scale.0], // x
        [0.0, center_y, 0.0, center_y / world_scale.0], // y
        [0.0, 0.0, 1.0, 0.0],                           // z
        [0.0, 0.0, 0.0, 1.0],                           // w
    ])
    .transpose();

    for (entity, ExtractedSyltWorldText { transform, .. }) in texts.iter() {
        let mut model_matrix =
            transform.compute_matrix().mul_scalar(world_scale.0);
        let mut view_matrix = view.world_from_view.compute_matrix();

        // flip the y-axis as Vello uses y-down and Bevy uses y-up
        model_matrix.w_axis.y *= -1.0;
        view_matrix.w_axis.y *= -1.0;

        // // TODO: scale
        // let scale_factor = svg.scale_factor;
        // model_matrix.x_axis.x *= scale_factor;
        // model_matrix.y_axis.y *= scale_factor;

        let projection_matrix = view.clip_from_view;
        let view_projection_matrix = projection_matrix * view_matrix.inverse();

        let model_render_transform =
            screen_matrix * view_projection_matrix * model_matrix;
        let transform = model_render_transform.to_cols_array();
        let transform: [f64; 6] = [
            transform[0] as f64,
            -transform[1] as f64,
            -transform[4] as f64,
            transform[5] as f64,
            transform[12] as f64,
            transform[13] as f64,
        ];
        cmd.entity(entity)
            .insert(PreparedTextAffine(vello::kurbo::Affine::new(transform)));
    }
}

#[derive(Component)]
struct ExtractedSyltWorldSvg {
    pub inner: SyltSvg,
    pub svg_anchor: SyltSvgAnchor,
    pub transform: GlobalTransform,
}

#[derive(Debug, Component)]
struct PreparedSvgAffine(pub vello::kurbo::Affine);

fn extract_sylt_svg(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        With<SyltWorldCamera>,
    >,

    svgs: Extract<
        Query<
            (
                &GlobalTransform,
                &SyltSvg,
                &SyltSvgAnchor,
                Option<&RenderLayers>,
            ),
            With<SyltWorldSvg>,
        >,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (transform, svg, svg_anchor, maybe_render_layers) in svgs.iter() {
        // Check if any camera renders this asset
        if views.iter().any(|(_camera, maybe_camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(maybe_camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltWorldSvg {
                    inner: svg.clone(),
                    svg_anchor: *svg_anchor,
                    transform: *transform,
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_svg_affines(
    mut cmd: Commands,
    views: Single<(&ExtractedCamera, &ExtractedView), With<SyltWorldCamera>>,
    svgs: Query<(Entity, &ExtractedSyltWorldSvg)>,
    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (camera, view) = *views;
    let viewport_dimension_in_pixels = camera.physical_viewport_size.unwrap();
    let (center_x, center_y) = (
        (viewport_dimension_in_pixels.x as f32 * 0.5),
        (viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    // 0,0 is the center of the 2d viewport in Bevy
    // so we need to transpose the matrix to have Vello
    // use the center of the viewport as 0,0 as well
    let screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [center_x, 0.0, 0.0, center_x / world_scale.0], // x
        [0.0, center_y, 0.0, center_y / world_scale.0], // y
        [0.0, 0.0, 1.0, 0.0],                           // z
        [0.0, 0.0, 0.0, 1.0],                           // w
    ])
    .transpose();

    for (
        entity,
        ExtractedSyltWorldSvg {
            inner: svg,
            transform,
            ..
        },
    ) in svgs.iter()
    {
        let mut model_matrix =
            transform.compute_matrix().mul_scalar(world_scale.0);
        let mut view_matrix = view.world_from_view.compute_matrix();

        // flip the y-axis as Vello uses y-down and Bevy uses y-up
        model_matrix.w_axis.y *= -1.0;
        view_matrix.w_axis.y *= -1.0;

        let scale_factor = svg.scale_factor;
        model_matrix.x_axis.x *= scale_factor;
        model_matrix.y_axis.y *= scale_factor;

        let projection_matrix = view.clip_from_view;
        let view_projection_matrix = projection_matrix * view_matrix.inverse();

        let model_render_transform =
            screen_matrix * view_projection_matrix * model_matrix;
        let transform = model_render_transform.to_cols_array();
        let transform: [f64; 6] = [
            transform[0] as f64,
            -transform[1] as f64,
            -transform[4] as f64,
            transform[5] as f64,
            transform[12] as f64,
            transform[13] as f64,
        ];
        cmd.entity(entity)
            .insert(PreparedSvgAffine(vello::kurbo::Affine::new(transform)));
    }
}

#[allow(clippy::too_many_arguments)]
fn render_world_space_canvas(
    render_target_texture: Res<WorldSpaceCanvasTexture>,
    renderer: Res<VelloRenderer>,

    // sylt_svgs: Res<SyltSvgCollection>,
    // svg_assets: Res<RenderAssets<SyltSvgAsset>>,

    // internals
    gpu_images: Res<RenderAssets<GpuImage>>,

    // wgpu
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    views: Query<(&ExtractedCamera, Option<&RenderLayers>)>,

    // renderables
    scenes: Query<(&PreparedSceneAffine, &ExtractedSyltScene)>,
    texts: Query<(&PreparedTextAffine, &ExtractedSyltWorldText)>,
    svgs: Query<(&PreparedSvgAffine, &ExtractedSyltWorldSvg)>,
) {
    enum RenderItem<'a> {
        Scene(&'a ExtractedSyltScene),
        Text(&'a ExtractedSyltWorldText),
        Svg(&'a ExtractedSyltWorldSvg),
    }

    let mut render_queue: Vec<(f32, vello::kurbo::Affine, RenderItem)> = scenes
        .iter()
        .map(|(affine, sylt_scene)| {
            let ExtractedSyltScene { transform, .. } = sylt_scene;

            (
                transform.translation().z,
                affine.0,
                RenderItem::Scene(sylt_scene),
            )
        })
        .collect();

    render_queue.extend(texts.iter().map(|(affine, sylt_text)| {
        let ExtractedSyltWorldText { transform, .. } = sylt_text;

        (
            transform.translation().z,
            affine.0,
            RenderItem::Text(sylt_text),
        )
    }));

    render_queue.extend(svgs.iter().map(|(affine, sylt_svg)| {
        let ExtractedSyltWorldSvg { transform, .. } = sylt_svg;

        (
            transform.translation().z,
            affine.0,
            RenderItem::Svg(sylt_svg),
        )
    }));

    render_queue.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut views: Vec<(&ExtractedCamera, Option<&RenderLayers>)> =
        views.into_iter().collect();
    views.sort_by(|(camera_a, _), (camera_b, _)| {
        camera_a.order.cmp(&camera_b.order)
    });

    let mut master_scene = vello::Scene::new();

    for (_, affine, render_item) in render_queue.into_iter() {
        match render_item {
            RenderItem::Scene(extracted_scene) => {
                master_scene.append(&extracted_scene.inner.inner, Some(affine));
            }
            RenderItem::Text(extracted_text) => {
                let text_align = &extracted_text.text_align;
                let text_anchor = &extracted_text.text_anchor;
                let text_style = &extracted_text.text_style;

                // TODO: Scaled
                extracted_text.inner.render(
                    &mut master_scene,
                    affine,
                    text_style,
                    text_anchor,
                    text_align,
                );
            }
            RenderItem::Svg(extracted_svg) => {
                continue;
                // let svg_anchor = &extracted_svg.svg_anchor;
                //
                // let svg_collection = match extracted_svg.inner.glyph {
                //     SyltSvgGlyph::None => continue,
                //     SyltSvgGlyph::SomeDefault => continue;
                // };
                //
                // // TODO: calculate scale in calculate affine
                // if let Some(svg_handle) = svg_collection {
                //     let svg = svg_assets.get(svg_handle).unwrap();
                //
                //     extracted_svg.inner.render(
                //         &mut master_scene,
                //         affine,
                //         svg,
                //         svg_anchor,
                //     );
                // }
            }
        }
    }

    let gpu_image = gpu_images.get(&render_target_texture.0).unwrap();

    let params = RenderParams {
        base_color: vello::peniko::color::palette::css::DARK_SLATE_BLUE,
        width: gpu_image.size.width,
        height: gpu_image.size.height,
        // TODO: config/settings
        antialiasing_method: vello::AaConfig::Area,
    };

    renderer
        .lock()
        .unwrap()
        .render_to_texture(
            device.wgpu_device(),
            &queue,
            &master_scene,
            &gpu_image.texture_view,
            &params,
        )
        .unwrap();
}

fn sylt_scene_picking(
    pointers: Query<(&bevy::picking::pointer::PointerId, &PointerLocation)>,
    camera: Single<
        (Entity, &Camera, &GlobalTransform, &Projection),
        With<SyltWorldCamera>,
    >,

    sylt_bez_paths_query: Query<(
        Entity,
        &SyltPickingShape,
        &GlobalTransform,
        Option<&Pickable>,
    )>,

    mut output: EventWriter<PointerHits>,

    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (cam_entity, camera, cam_transform, cam_ortho) = *camera;
    if let Projection::Orthographic(cam_ortho) = cam_ortho {
        let mut sorted_bez_paths: Vec<_> = sylt_bez_paths_query
            .iter()
            .filter_map(|(entity, bez_path, transform, picking_behavior)| {
                if !transform.affine().is_nan() {
                    Some((entity, bez_path, transform, picking_behavior))
                } else {
                    None
                }
            })
            .collect();

        sorted_bez_paths
            .sort_by_key(|x| Reverse(FloatOrd(x.2.translation().z)));

        let world_scale_matrix = Mat4::from_scale_rotation_translation(
            Vec2::splat(world_scale.0).extend(1.0),
            Quat::IDENTITY,
            Vec3::ZERO,
        );

        let cam_transform = GlobalTransform::from(Transform::from_matrix(
            cam_transform.affine() * world_scale_matrix.inverse(),
        ));

        for (pointer, location) in pointers.iter() {
            let maybe_location = location.location();
            if maybe_location.is_none() {
                continue;
            }
            let pointer_location = maybe_location.unwrap();

            let mut blocked = false;

            let viewport_pos = camera
                .logical_viewport_rect()
                .map(|v| v.min)
                .unwrap_or_default();

            let pos_in_viewport = pointer_location.position - viewport_pos;

            let Ok(cursor_ray_world) =
                camera.viewport_to_world(&cam_transform, pos_in_viewport)
            else {
                continue;
            };

            let cursor_ray_len = cam_ortho.far - cam_ortho.near;
            let cursor_ray_end = cursor_ray_world.origin
                + *cursor_ray_world.direction * cursor_ray_len;

            let picks: Vec<(Entity, HitData)> = sorted_bez_paths
                .iter()
                .copied()
                .filter_map(
                    |(
                        entity,
                        bez_path,
                        bez_path_transform,
                        picking_behavior,
                    )| {
                        if blocked {
                            return None;
                        }

                        let world_to_bez_path =
                            bez_path_transform.affine().inverse();

                        let cursor_start = world_to_bez_path
                            .transform_point3(cursor_ray_world.origin);
                        let cursor_end =
                            world_to_bez_path.transform_point3(cursor_ray_end);

                        let lerp_factor = f32::inverse_lerp(
                            cursor_start.z,
                            cursor_end.z,
                            0.0,
                        );
                        let cursor_pos_in_bez_path =
                            cursor_start.lerp(cursor_end, lerp_factor);

                        let is_cursor_in_bez_path = bez_path.inner.contains(
                            vello::kurbo::Point::new(
                                cursor_pos_in_bez_path.x as f64,
                                -cursor_pos_in_bez_path.y as f64,
                            ) + bez_path.affine.inverse().translation(),
                        );

                        blocked = is_cursor_in_bez_path
                            && picking_behavior
                                .map(|p| p.should_block_lower)
                                .unwrap_or(true);

                        is_cursor_in_bez_path.then(|| {
                            let hit_pos_world = bez_path_transform
                                .transform_point(pos_in_viewport.extend(0.0));
                            let hit_pos_cam = cam_transform
                                .affine()
                                .inverse()
                                .transform_point3(hit_pos_world);

                            let depth = -cam_ortho.near - hit_pos_cam.z;
                            (
                                entity,
                                HitData::new(
                                    cam_entity,
                                    depth,
                                    Some(hit_pos_world),
                                    Some(*bez_path_transform.back()),
                                ),
                            )
                        })
                    },
                )
                .collect();

            let order = camera.order as f32;
            output.write(PointerHits::new(*pointer, picks, order));
        }
    }
}
