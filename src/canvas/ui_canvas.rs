use bevy::{
    asset::{load_internal_asset, weak_handle, RenderAssetUsages},
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
    ui::ContentSize,
    window::{PrimaryWindow, WindowResized},
};
use vello::{
    wgpu::{
        Extent3d, PrimitiveTopology, TextureDescriptor, TextureDimension,
        TextureFormat, TextureUsages, VertexFormat, VertexStepMode,
    },
    RenderParams,
};

use crate::{
    cameras::{SyltUiCamera, SyltWorldCamera},
    canvas::world_canvas::SyltWorldCanvasScale,
    settings::SyltSettings,
};

use super::{
    svg::{
        SyltSvg, SyltSvgAnchor, SyltSvgAsset, SyltSvgCollection, SyltSvgGlyph,
    },
    text::{SyltText, SyltTextAlign, SyltTextAnchor, SyltTextStyle},
    VelloRenderer,
};

pub struct SyltUiCanvasPlugin;

impl Plugin for SyltUiCanvasPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            UI_SHADER_HANDLE,
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
            (render_screen_space_canvas)
                .chain()
                .in_set(RenderSet::Render)
                .run_if(resource_exists::<RenderDevice>),
        );

        app.register_type::<ScreenSpaceCanvasTexture>();
        app.register_type::<ScreenSpaceCanvas>();

        app.insert_resource(SyltUiCanvasScale::default());
        app.insert_resource(SyltUiCanvasScaleFactor(1.0));
        app.add_plugins(ExtractResourcePlugin::<SyltUiCanvasScale>::default());

        app.add_plugins(
            Material2dPlugin::<ScreenSpaceCanvasMaterial>::default(),
        );

        // Screen Canvas (for rendering)
        app.add_plugins(
            ExtractResourcePlugin::<ScreenSpaceCanvasTexture>::default(),
        );
        app.add_systems(Startup, setup_screen_space_canvas_mesh);
        app.add_systems(
            Update,
            (
                resize_screen_space_canvas,
                set_ui_scale_factor_on_window_resize_system,
                set_ui_scale_on_ui_scale_setting_change_system,
                propagate_ui_scale_to_bevy_system,
            ),
        );
    }
}

#[derive(Debug, Clone, Component)]
pub struct NoSyltUiScaling;

#[derive(Debug, Clone, Component)]
pub struct SyltUiUseWorldCoorindates;

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct SyltUiCanvasScale(pub f32);

#[derive(Resource, Clone, Debug, Reflect)]
pub struct SyltUiCanvasScaleFactor(pub f32);

const DEFAULT_WINDOW_WIDTH: f32 = 1280.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 720.0;

fn set_ui_scale_factor_on_window_resize_system(
    mut cmd: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut window_resized_events: EventReader<WindowResized>,
) {
    if window_resized_events.read().last().is_some() {
        let max_width = window.physical_width() as f32;
        let max_height = window.physical_height() as f32;
        let width = DEFAULT_WINDOW_WIDTH;
        let height = DEFAULT_WINDOW_HEIGHT;
        let size = Vec2::new(width, height);
        let max_size = Vec2::new(max_width, max_height);
        let scale_factor = max_size / size;
        let scale_factor = scale_factor.x.max(scale_factor.y);

        cmd.insert_resource(SyltUiCanvasScaleFactor(scale_factor));
        cmd.insert_resource(SyltUiCanvasScale(
            scale_factor * SyltSettings::default().ui_scale.0,
        ));
    }
}

fn set_ui_scale_on_ui_scale_setting_change_system(
    mut cmd: Commands,
    settings: Res<SyltSettings>,
    scale_factor: Res<SyltUiCanvasScaleFactor>,
) {
    if settings.is_changed() {
        cmd.insert_resource(SyltUiCanvasScale(
            settings.ui_scale.0 * scale_factor.0,
        ));
    }
}

fn propagate_ui_scale_to_bevy_system(
    mut cmd: Commands,
    ui_scale: Res<SyltUiCanvasScale>,
    ui_scale_factor: Res<SyltUiCanvasScaleFactor>,
) {
    if ui_scale.is_changed() {
        cmd.insert_resource(UiScale(ui_scale.0 * ui_scale_factor.0));
    }
}

impl Default for SyltUiCanvasScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl ExtractResource for SyltUiCanvasScale {
    type Source = SyltUiCanvasScale;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

/// Transform y-down
/// Draw y-down
#[derive(Default, Clone, Component)]
#[require(Transform, ZIndex)]
pub struct SyltUiScene {
    pub inner: vello::Scene,
    pub pixel_size: Vec2,
}

impl SyltUiScene {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<vello::Scene> for SyltUiScene {
    fn from(scene: vello::Scene) -> Self {
        Self {
            inner: scene,
            pixel_size: Vec2::new(0.0, 0.0),
        }
    }
}

/// Transform y-down
#[derive(Default, Clone, Component)]
#[require(Transform, ZIndex, ContentSize)]
pub struct SyltUiText;

/// Transform y-down
#[derive(Clone, Component)]
#[require(Transform, ZIndex, ContentSize)]
pub struct SyltUiSvg;

const UI_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("738a6b3b-6437-454a-ba63-6fff49491144");

// TODO: Asset should probably be a resource
#[derive(Reflect, Component)]
#[reflect(Component)]
struct ScreenSpaceCanvas(pub Handle<ScreenSpaceCanvasMaterial>);

#[derive(Reflect, Resource, Default, Clone)]
#[reflect(Resource)]
struct ScreenSpaceCanvasTexture(pub Handle<Image>);

impl ExtractResource for ScreenSpaceCanvasTexture {
    type Source = ScreenSpaceCanvasTexture;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

#[derive(AsBindGroup, TypePath, Asset, Clone)]
struct ScreenSpaceCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for ScreenSpaceCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        UI_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        UI_SHADER_HANDLE.into()
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

fn setup_screen_space_canvas_mesh(
    mut cmd: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut canvas_materials: ResMut<Assets<ScreenSpaceCanvasMaterial>>,
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
            label: Some("SyltUiCanvasTexture"),
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
    let screen_space_canvas_texture =
        ScreenSpaceCanvasTexture(image_handle.clone());
    cmd.insert_resource(screen_space_canvas_texture);

    // 2D Mesh Material
    let material_handle = canvas_materials.add(ScreenSpaceCanvasMaterial {
        texture: image_handle,
    });

    cmd.spawn((
        Name::new("SyltUiCanvas"),
        ScreenSpaceCanvas(material_handle.clone()),
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(material_handle),
        NoFrustumCulling,
        RenderLayers::layer(1),
    ));
}

fn resize_screen_space_canvas(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    mut window_resized_events: EventReader<WindowResized>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut canvas: Query<&mut ScreenSpaceCanvas>,
    mut canvas_texture: ResMut<ScreenSpaceCanvasTexture>,
    mut canvas_material_assets: ResMut<Assets<ScreenSpaceCanvasMaterial>>,
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

        for material in canvas.iter_mut() {
            let mut texture_image = Image {
                texture_descriptor: TextureDescriptor {
                    label: Some("ScreenSpaceCanvas"),
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
            let screen_space_canvas_texture =
                ScreenSpaceCanvasTexture(texture_image_handle.clone());
            cmd.insert_resource(screen_space_canvas_texture.clone());

            canvas_texture.0 = texture_image_handle.clone();
            canvas_material_assets.get_mut(&material.0).unwrap().texture =
                texture_image_handle;
        }
    }
}

#[derive(Component)]
struct ExtractedSyltScene {
    pub inner: vello::Scene,
    pub transform: GlobalTransform,
    pub z_index: ZIndex,
    pub maybe_computed_node: Option<ComputedNode>,
    pub maybe_no_ui_scaling: Option<NoSyltUiScaling>,
    pub maybe_use_world_coordinates: Option<SyltUiUseWorldCoorindates>,
}

#[derive(Debug, Component)]
struct PreparedSceneAffine(pub vello::kurbo::Affine);

fn extract_sylt_scene(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<SyltUiCamera>, With<Camera2d>),
    >,

    scenes: Extract<
        Query<(
            &GlobalTransform,
            &ZIndex,
            &SyltUiScene,
            Option<&ComputedNode>,
            Option<&RenderLayers>,
            Option<&NoSyltUiScaling>,
            Option<&SyltUiUseWorldCoorindates>,
        )>,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        transform,
        z_index,
        scene,
        maybe_computed_node,
        maybe_render_layers,
        maybe_no_ui_scaling,
        maybe_use_world_coordinates,
    ) in scenes.iter()
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
                    z_index: *z_index,
                    inner: scene.inner.clone(),
                    maybe_computed_node: maybe_computed_node.cloned(),
                    maybe_no_ui_scaling: maybe_no_ui_scaling.cloned(),
                    maybe_use_world_coordinates: maybe_use_world_coordinates
                        .cloned(),
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_scene_affines(
    mut cmd: Commands,
    views: Single<(&ExtractedCamera, &ExtractedView), With<SyltUiCamera>>,
    scenes: Query<(Entity, &ExtractedSyltScene)>,
    ui_scale: Res<SyltUiCanvasScale>,

    world_views: Single<
        (&ExtractedCamera, &ExtractedView),
        With<SyltWorldCamera>,
    >,
    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (camera, view) = *views;
    let viewport_dimension_in_pixels = camera.physical_viewport_size.unwrap();
    let (center_x, center_y) = (
        (viewport_dimension_in_pixels.x as f32 * 0.5),
        (viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    let screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [center_x, 0.0, 0.0, center_x], // x
        [0.0, center_y, 0.0, center_y], // y
        [0.0, 0.0, 1.0, 0.0],           // z
        [0.0, 0.0, 0.0, 1.0],           // w
    ]);

    // Do not transpose the matrix, Bevy UI and Vello both have 0,0 in the top left
    //
    let (world_camera, world_view) = *world_views;
    let world_viewport_dimension_in_pixels =
        world_camera.physical_viewport_size.unwrap();
    let (world_center_x, world_center_y) = (
        (world_viewport_dimension_in_pixels.x as f32 * 0.5),
        (world_viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    let world_screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [world_center_x, 0.0, 0.0, world_center_x / world_scale.0], // x
        [0.0, world_center_y, 0.0, world_center_y / world_scale.0], // y
        [0.0, 0.0, 1.0, 0.0],                                       // z
        [0.0, 0.0, 0.0, 1.0],                                       // w
    ])
    .transpose();

    for (
        entity,
        ExtractedSyltScene {
            transform,
            maybe_computed_node,
            maybe_no_ui_scaling,
            maybe_use_world_coordinates,
            ..
        },
    ) in scenes.iter()
    {
        if maybe_use_world_coordinates.is_some() {
            let mut model_matrix =
                transform.compute_matrix().mul_scalar(world_scale.0);
            let mut view_matrix = world_view.world_from_view.compute_matrix();

            // flip the y-axis as Vello uses y-down and Bevy uses y-up
            model_matrix.w_axis.y *= -1.0;
            view_matrix.w_axis.y *= -1.0;

            let projection_matrix = world_view.clip_from_view;
            let view_projection_matrix =
                projection_matrix * view_matrix.inverse();

            let model_render_transform =
                world_screen_matrix * view_projection_matrix * model_matrix;
            let transform = model_render_transform.to_cols_array();
            let transform: [f64; 6] = [
                transform[0] as f64,
                -transform[1] as f64,
                -transform[4] as f64,
                transform[5] as f64,
                transform[12] as f64,
                transform[13] as f64,
            ];
            cmd.entity(entity).insert(PreparedSceneAffine(
                vello::kurbo::Affine::new(transform),
            ));
        } else {
            let model_render_transform =
                if let Some(computed_node) = maybe_computed_node {
                    let mut model_matrix = transform.compute_matrix();
                    let Vec2 { x, y } = computed_node.size();
                    model_matrix.w_axis.x -= x * 0.5;
                    model_matrix.w_axis.y -= y * 0.5;
                    model_matrix
                } else {
                    let mut model_matrix = transform.compute_matrix();
                    let view_matrix = view.world_from_view.compute_matrix();
                    if maybe_no_ui_scaling.is_none() {
                        model_matrix.x_axis.x *= ui_scale.0;
                        model_matrix.y_axis.y *= ui_scale.0;
                    }
                    let projection_matrix = view.clip_from_view;
                    let view_projection_matrix =
                        projection_matrix * view_matrix.inverse();
                    screen_matrix * view_projection_matrix * model_matrix
                };

            let transform = model_render_transform.to_cols_array();
            let transform: [f64; 6] = [
                transform[0] as f64,
                -transform[1] as f64,
                -transform[4] as f64,
                transform[5] as f64,
                transform[12] as f64,
                transform[13] as f64,
            ];
            cmd.entity(entity).insert(PreparedSceneAffine(
                vello::kurbo::Affine::new(transform),
            ));
        }
    }
}

#[derive(Component)]
struct ExtractedSyltUiText {
    pub inner: SyltText,
    pub text_style: SyltTextStyle,
    pub text_align: SyltTextAlign,
    pub text_anchor: SyltTextAnchor,
    pub transform: GlobalTransform,
    pub z_index: ZIndex,
    pub maybe_no_ui_scaling: Option<NoSyltUiScaling>,
    pub maybe_use_world_coordinates: Option<SyltUiUseWorldCoorindates>,
}

#[derive(Debug, Component)]
struct PreparedTextAffine(pub vello::kurbo::Affine);

fn extract_sylt_text(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<SyltUiCamera>, With<Camera2d>),
    >,

    texts: Extract<
        Query<
            (
                &GlobalTransform,
                &ZIndex,
                &SyltText,
                &SyltTextStyle,
                &SyltTextAnchor,
                &SyltTextAlign,
                Option<&RenderLayers>,
                Option<&NoSyltUiScaling>,
                Option<&SyltUiUseWorldCoorindates>,
            ),
            With<SyltUiText>,
        >,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        transform,
        z_index,
        text,
        text_style,
        text_anchor,
        text_align,
        maybe_render_layers,
        maybe_no_ui_scaling,
        maybe_use_world_coordinates,
    ) in texts.iter()
    {
        // Check if any camera renders this asset
        if views.iter().any(|(_camera, maybe_camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(maybe_camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltUiText {
                    inner: text.clone(),
                    text_style: text_style.clone(),
                    text_align: *text_align,
                    text_anchor: *text_anchor,
                    transform: *transform,
                    z_index: *z_index,
                    maybe_no_ui_scaling: maybe_no_ui_scaling.cloned(),
                    maybe_use_world_coordinates: maybe_use_world_coordinates
                        .cloned(),
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_text_affines(
    mut cmd: Commands,
    texts: Query<(Entity, &ExtractedSyltUiText)>,
    ui_scale: Res<SyltUiCanvasScale>,

    world_views: Single<
        (&ExtractedCamera, &ExtractedView),
        With<SyltWorldCamera>,
    >,
    world_scale: Res<SyltWorldCanvasScale>,
) {
    let (world_camera, world_view) = *world_views;
    let world_viewport_dimension_in_pixels =
        world_camera.physical_viewport_size.unwrap();
    let (world_center_x, world_center_y) = (
        (world_viewport_dimension_in_pixels.x as f32 * 0.5),
        (world_viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    let world_screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [world_center_x, 0.0, 0.0, world_center_x / world_scale.0], // x
        [0.0, world_center_y, 0.0, world_center_y / world_scale.0], // y
        [0.0, 0.0, 1.0, 0.0],                                       // z
        [0.0, 0.0, 0.0, 1.0],                                       // w
    ])
    .transpose();

    for (
        entity,
        ExtractedSyltUiText {
            transform,
            maybe_no_ui_scaling,
            maybe_use_world_coordinates,
            ..
        },
    ) in texts.iter()
    {
        if maybe_use_world_coordinates.is_some() {
            let mut model_matrix =
                transform.compute_matrix().mul_scalar(world_scale.0);
            let mut view_matrix = world_view.world_from_view.compute_matrix();

            // flip the y-axis as Vello uses y-down and Bevy uses y-up
            model_matrix.w_axis.y *= -1.0;
            view_matrix.w_axis.y *= -1.0;

            let projection_matrix = world_view.clip_from_view;
            let view_projection_matrix =
                projection_matrix * view_matrix.inverse();

            let model_render_transform =
                world_screen_matrix * view_projection_matrix * model_matrix;
            let transform = model_render_transform.to_cols_array();
            let transform: [f64; 6] = [
                transform[0] as f64,
                -transform[1] as f64,
                -transform[4] as f64,
                transform[5] as f64,
                transform[12] as f64,
                transform[13] as f64,
            ];
            cmd.entity(entity).insert(PreparedTextAffine(
                vello::kurbo::Affine::new(transform),
            ));
        } else {
            let mut model_matrix = transform.compute_matrix();
            if maybe_no_ui_scaling.is_none() {
                model_matrix.x_axis.x *= ui_scale.0;
                model_matrix.y_axis.y *= ui_scale.0;
            }
            let transform = model_matrix.to_cols_array();
            let transform: [f64; 6] = [
                transform[0] as f64,
                -transform[1] as f64,
                -transform[4] as f64,
                transform[5] as f64,
                transform[12] as f64,
                transform[13] as f64,
            ];
            cmd.entity(entity).insert(PreparedTextAffine(
                vello::kurbo::Affine::new(transform),
            ));
        }
    }
}

#[derive(Component)]
struct ExtractedSyltUiSvg {
    pub inner: SyltSvg,
    pub svg_anchor: SyltSvgAnchor,
    pub transform: GlobalTransform,
    pub z_index: ZIndex,
    pub maybe_computed_node: Option<ComputedNode>,
    pub maybe_no_ui_scaling: Option<NoSyltUiScaling>,
    pub maybe_use_world_coordinates: Option<SyltUiUseWorldCoorindates>,
}

#[derive(Debug, Component)]
struct PreparedSvgAffine(pub vello::kurbo::Affine);

fn extract_sylt_svg(
    mut cmd: Commands,

    view_q: Query<
        (&ExtractedCamera, Option<&RenderLayers>),
        (With<SyltUiCamera>, With<Camera2d>),
    >,

    svgs: Extract<
        Query<
            (
                &GlobalTransform,
                &ZIndex,
                &SyltSvg,
                &SyltSvgAnchor,
                Option<&ComputedNode>,
                Option<&RenderLayers>,
                Option<&NoSyltUiScaling>,
                Option<&SyltUiUseWorldCoorindates>,
            ),
            With<SyltUiSvg>,
        >,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (
        transform,
        z_index,
        svg,
        svg_anchor,
        maybe_computed_node,
        maybe_render_layers,
        maybe_no_ui_scaling,
        maybe_use_world_coordinates,
    ) in svgs.iter()
    {
        // Check if any camera renders this asset
        if views.iter().any(|(_camera, maybe_camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(maybe_camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltUiSvg {
                    inner: svg.clone(),
                    svg_anchor: *svg_anchor,
                    transform: *transform,
                    z_index: *z_index,
                    maybe_computed_node: maybe_computed_node.cloned(),
                    maybe_no_ui_scaling: maybe_no_ui_scaling.cloned(),
                    maybe_use_world_coordinates: maybe_use_world_coordinates
                        .cloned(),
                },
                TemporaryRenderEntity,
            ));
        }
    }
}

fn prepare_svg_affines(
    mut cmd: Commands,
    views: Single<(&ExtractedCamera, &ExtractedView), With<SyltUiCamera>>,
    svgs: Query<(Entity, &ExtractedSyltUiSvg)>,
    ui_scale: Res<SyltUiCanvasScale>,
) {
    let (camera, view) = *views;
    let viewport_dimension_in_pixels = camera.physical_viewport_size.unwrap();
    let (center_x, center_y) = (
        (viewport_dimension_in_pixels.x as f32 * 0.5),
        (viewport_dimension_in_pixels.y as f32 * 0.5),
    );
    // Do not transpose the matrix, Bevy UI and Vello both have 0,0 in the top left
    let screen_matrix = Mat4::from_cols_array_2d(&[
        // column 1-3 scales the NDC to the viewport size
        // column 4 translates the NDC to the center of the viewport
        [center_x, 0.0, 0.0, center_x], // x
        [0.0, center_y, 0.0, center_y], // y
        [0.0, 0.0, 1.0, 0.0],           // z
        [0.0, 0.0, 0.0, 1.0],           // w
    ]);

    for (
        entity,
        ExtractedSyltUiSvg {
            transform,
            maybe_computed_node,
            maybe_no_ui_scaling,
            inner: svg,
            ..
        },
    ) in svgs.iter()
    {
        let scale_factor = svg.scale_factor;

        let model_render_transform = if maybe_computed_node.is_some() {
            let mut model_matrix = transform.compute_matrix();
            model_matrix.x_axis.x *= scale_factor;
            model_matrix.y_axis.y *= scale_factor;
            if maybe_no_ui_scaling.is_none() {
                model_matrix.x_axis.x *= ui_scale.0;
                model_matrix.y_axis.y *= ui_scale.0;
            }
            model_matrix
        } else {
            let mut model_matrix = transform.compute_matrix();
            let view_matrix = view.world_from_view.compute_matrix();
            model_matrix.x_axis.x *= scale_factor;
            model_matrix.y_axis.y *= scale_factor;
            if maybe_no_ui_scaling.is_none() {
                model_matrix.x_axis.x *= ui_scale.0;
                model_matrix.y_axis.y *= ui_scale.0;
            }
            let projection_matrix = view.clip_from_view;
            let view_projection_matrix =
                projection_matrix * view_matrix.inverse();
            screen_matrix * view_projection_matrix * model_matrix
        };

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
fn render_screen_space_canvas(
    render_target_texture: Res<ScreenSpaceCanvasTexture>,
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
    texts: Query<(&PreparedTextAffine, &ExtractedSyltUiText)>,
    svgs: Query<(&PreparedSvgAffine, &ExtractedSyltUiSvg)>,
) {
    enum RenderItem<'a> {
        Scene(&'a ExtractedSyltScene),
        Text(&'a ExtractedSyltUiText),
        Svg(&'a ExtractedSyltUiSvg),
    }

    let mut render_queue: Vec<(i32, vello::kurbo::Affine, RenderItem)> = scenes
        .iter()
        .map(|(affine, sylt_scene)| {
            let ExtractedSyltScene { z_index, .. } = sylt_scene;

            (z_index.0, affine.0, RenderItem::Scene(sylt_scene))
        })
        .collect();

    render_queue.extend(texts.iter().map(|(affine, sylt_text)| {
        let ExtractedSyltUiText { z_index, .. } = sylt_text;

        (z_index.0, affine.0, RenderItem::Text(sylt_text))
    }));

    render_queue.extend(svgs.iter().map(|(affine, sylt_svg)| {
        let ExtractedSyltUiSvg { z_index, .. } = sylt_svg;

        (z_index.0, affine.0, RenderItem::Svg(sylt_svg))
    }));

    render_queue.sort_by(|a, b| a.0.cmp(&b.0));

    let mut views: Vec<(&ExtractedCamera, Option<&RenderLayers>)> =
        views.into_iter().collect();
    views.sort_by(|(camera_a, _), (camera_b, _)| {
        camera_a.order.cmp(&camera_b.order)
    });

    let mut master_scene = vello::Scene::new();

    for (_, affine, render_item) in render_queue.into_iter() {
        match render_item {
            RenderItem::Scene(extracted_scene) => {
                master_scene.append(&extracted_scene.inner, Some(affine));
            }
            RenderItem::Text(extracted_text) => {
                let text_align = &extracted_text.text_align;
                let text_anchor = &extracted_text.text_anchor;
                let text_style = &extracted_text.text_style;

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
                //     // SyltSvgGlyph::Generic(glyph) => {
                //     //     sylt_svgs.generic.get(&glyph)
                //     // }
                //     // SyltSvgGlyph::KeyboardAndMouse(glyph) => {
                //     //     sylt_svgs.keyboard_and_mouse.get(&glyph)
                //     // }
                //     // SyltSvgGlyph::PlayStation(glyph) => {
                //     //     sylt_svgs.playstation.get(&glyph)
                //     // }
                //     // SyltSvgGlyph::SteamDeck(glyph) => {
                //     //     sylt_svgs.steam_deck.get(&glyph)
                //     // }
                //     // SyltSvgGlyph::Switch(glyph) => sylt_svgs.switch.get(&glyph),
                //     // SyltSvgGlyph::Xbox(glyph) => sylt_svgs.xbox.get(&glyph),
                // };
                //
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
        base_color: vello::peniko::color::palette::css::TRANSPARENT,
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
