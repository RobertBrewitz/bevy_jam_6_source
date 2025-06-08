use bevy::{
    image::ImageSampler,
    math::Affine2,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
        renderer::{RenderDevice, RenderQueue},
        sync_world::TemporaryRenderEntity,
        texture::GpuImage,
        view::{ExtractedView, RenderLayers},
        Extract, Render, RenderApp, RenderSet,
    },
};
use vello::{
    kurbo::Affine,
    wgpu::{TextureDescriptor, TextureFormat, TextureUsages},
    RenderParams,
};

use super::{
    svg::{
        SyltSvg, SyltSvgAnchor, SyltSvgAsset, SyltSvgCollection, SyltSvgGlyph,
    },
    text::{SyltText, SyltTextAlign, SyltTextAnchor, SyltTextStyle},
    VelloRenderer,
};

pub struct SyltMeshCanvasPlugin;

// TODO: get widest edge, use that to "fill_scale" mesh canvas
// TODO: 3D scaling or just move camera?

impl Plugin for SyltMeshCanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_add_mesh_canvas);

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(ExtractSchedule, extract_sylt_mesh_canvas);
        render_app
            .add_systems(Render, prepare_affines.in_set(RenderSet::Prepare));
        // TODO: mesh texture atlas
        render_app.add_systems(
            Render,
            render_sylt_mesh_scene
                .in_set(RenderSet::Render)
                .run_if(resource_exists::<RenderDevice>),
        );
    }
}

#[derive(Component, Clone)]
#[require(Transform)]
pub struct SyltMeshCanvas {
    pub handle: Handle<Image>,
    pub pixel_size: Vec2,
    pub base_color: vello::peniko::Color,
}

impl Default for SyltMeshCanvas {
    fn default() -> Self {
        Self {
            handle: Default::default(),
            pixel_size: Vec2::new(256.0, 256.0),
            base_color: vello::peniko::Color::TRANSPARENT,
        }
    }
}

#[derive(Default, Component, Clone)]
#[require(Transform)]
pub struct SyltMeshScene {
    pub inner: vello::Scene,
    pub pixel_size: Vec2,
}

#[derive(Default, Clone, Component)]
#[require(Transform)]
pub struct SyltMeshText;

#[derive(Default, Component, Clone)]
#[require(Transform)]
pub struct SyltMeshSvg;

fn on_add_mesh_canvas(
    trigger: Trigger<OnAdd, SyltMeshCanvas>,
    mut mesh_canvas_q: Query<(Entity, &mut SyltMeshCanvas)>,
    mut cmd: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (entity, mut mesh_canvas) =
        mesh_canvas_q.get_mut(trigger.target()).unwrap();

    // TODO: get size from scene?
    let image_size = vello::wgpu::Extent3d {
        width: mesh_canvas.pixel_size.x as u32,
        height: mesh_canvas.pixel_size.y as u32,
        depth_or_array_layers: 1,
    };

    // Trigger SyltMeshCanvas to generate a texture for the mesh based on size of scene
    let mut texture_image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("SyltMeshCanvasTexture"),
            size: image_size,
            dimension: vello::wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        },
        sampler: ImageSampler::linear(),
        ..default()
    };
    texture_image.resize(image_size);
    let texture_handle = images.add(texture_image);

    let mut entity_command = cmd.entity(entity);
    entity_command.insert((MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        // TODO: from scene property
        unlit: false,
        reflectance: 0.02,
        // TODO: calculate ppu from screen and projection
        uv_transform: Affine2::from_scale(Vec2::splat(1.0)),
        ..Default::default()
    })),));
    mesh_canvas.handle = texture_handle;
}

#[derive(Component, Clone)]
struct ExtractedSyltMeshCanvas {
    pub inner: SyltMeshCanvas,
}

#[derive(Component, Clone)]
struct ExtractedSyltMeshScene {
    pub inner: SyltMeshScene,
    pub transform: Transform,
}

#[derive(Component, Clone)]
struct ExtractedSyltMeshText {
    pub inner: SyltText,
    pub text_style: SyltTextStyle,
    pub text_align: SyltTextAlign,
    pub text_anchor: SyltTextAnchor,
    pub transform: Transform,
}

#[derive(Component, Clone)]
struct ExtractedSyltMeshSvg {
    pub inner: SyltSvg,
    pub transform: Transform,
    pub svg_anchor: SyltSvgAnchor,
}

#[allow(clippy::too_many_arguments)]
fn extract_sylt_mesh_canvas(
    mut cmd: Commands,

    view_q: Query<(&ExtractedCamera, Option<&RenderLayers>), With<Camera3d>>,

    mesh_canvas_q: Extract<
        Query<(&SyltMeshCanvas, &Children, Option<&RenderLayers>)>,
    >,
    mesh_scene_q: Extract<Query<(&SyltMeshScene, &Transform)>>,
    mesh_text_q: Extract<
        Query<
            (
                &SyltText,
                &SyltTextStyle,
                &SyltTextAlign,
                &SyltTextAnchor,
                &Transform,
            ),
            With<SyltMeshText>,
        >,
    >,
    mesh_svg_q: Extract<
        Query<(&SyltSvg, &SyltSvgAnchor, &Transform), With<SyltMeshSvg>>,
    >,
) {
    let mut views: Vec<_> = view_q.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (canvas, children, maybe_render_layers) in mesh_canvas_q.iter() {
        if views.iter().any(|(_camera, camera_layers)| {
            maybe_render_layers
                .unwrap_or_default()
                .intersects(camera_layers.unwrap_or_default())
        }) {
            cmd.spawn((
                ExtractedSyltMeshCanvas {
                    inner: canvas.clone(),
                },
                TemporaryRenderEntity,
            ))
            .with_children(|parent| {
                for child in children.iter() {
                    if let Ok((scene, transform)) = mesh_scene_q.get(child) {
                        parent.spawn((
                            ExtractedSyltMeshScene {
                                inner: scene.clone(),
                                transform: *transform,
                            },
                            TemporaryRenderEntity,
                        ));
                    }

                    if let Ok((
                        text,
                        text_style,
                        text_align,
                        text_anchor,
                        transform,
                    )) = mesh_text_q.get(child)
                    {
                        parent.spawn((
                            ExtractedSyltMeshText {
                                inner: text.clone(),
                                text_style: text_style.clone(),
                                text_align: *text_align,
                                text_anchor: *text_anchor,
                                transform: *transform,
                            },
                            TemporaryRenderEntity,
                        ));
                    }

                    if let Ok((svg, svg_anchor, transform)) =
                        mesh_svg_q.get(child)
                    {
                        parent.spawn((
                            ExtractedSyltMeshSvg {
                                inner: svg.clone(),
                                transform: *transform,
                                svg_anchor: *svg_anchor,
                            },
                            TemporaryRenderEntity,
                        ));
                    }
                }
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_affines(
    mut cmd: Commands,

    views: Query<(&ExtractedCamera, &ExtractedView), With<SyltMeshCanvas>>,

    mesh_canvas_q: Query<(&ExtractedSyltMeshCanvas, &Children)>,
    mesh_scene_q: Query<&ExtractedSyltMeshScene>,
    mesh_text_q: Query<&ExtractedSyltMeshText>,
    mesh_svg_q: Query<&ExtractedSyltMeshSvg>,
) {
    let mut views: Vec<_> = views.iter().collect();
    views.sort_unstable_by_key(|(camera, _)| camera.order);

    for (_mesh_canvas, children) in mesh_canvas_q.iter() {
        for child in children.iter() {
            if let Ok(mesh_scene) = mesh_scene_q.get(child) {
                let mut world_matrix = mesh_scene.transform.compute_matrix();
                world_matrix.w_axis.y *= -1.0;
                let transform: [f32; 16] = world_matrix.to_cols_array();
                let transform: [f64; 6] = [
                    transform[0] as f64,
                    -transform[1] as f64,
                    -transform[4] as f64,
                    transform[5] as f64,
                    transform[12] as f64,
                    transform[13] as f64,
                ];

                cmd.entity(child)
                    .insert(PreparedAffine(Affine::new(transform)));
            }
            if let Ok(mesh_text) = mesh_text_q.get(child) {
                let mut world_matrix = mesh_text.transform.compute_matrix();
                world_matrix.w_axis.y *= -1.0;
                let transform: [f32; 16] = world_matrix.to_cols_array();
                let transform: [f64; 6] = [
                    transform[0] as f64,
                    -transform[1] as f64,
                    -transform[4] as f64,
                    transform[5] as f64,
                    transform[12] as f64,
                    transform[13] as f64,
                ];

                cmd.entity(child)
                    .insert(PreparedAffine(Affine::new(transform)));
            }
            if let Ok(mesh_svg) = mesh_svg_q.get(child) {
                let svg = &mesh_svg.inner;
                let mut world_matrix = mesh_svg.transform.compute_matrix();
                world_matrix.w_axis.y *= -1.0;
                let scale_factor = svg.scale_factor;
                world_matrix.x_axis.x *= scale_factor;
                world_matrix.y_axis.y *= scale_factor;
                let transform: [f32; 16] = world_matrix.to_cols_array();
                let transform: [f64; 6] = [
                    transform[0] as f64,
                    -transform[1] as f64,
                    -transform[4] as f64,
                    transform[5] as f64,
                    transform[12] as f64,
                    transform[13] as f64,
                ];

                cmd.entity(child)
                    .insert(PreparedAffine(Affine::new(transform)));
            }
        }
    }
}

#[derive(Component, Clone, Deref)]
struct PreparedAffine(pub Affine);

#[allow(clippy::too_many_arguments)]
fn render_sylt_mesh_scene(
    renderer: Res<VelloRenderer>,
    device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    queue: Res<RenderQueue>,

    // sylt_svgs: Res<SyltSvgCollection>,
    // svg_assets: Res<RenderAssets<SyltSvgAsset>>,
    mesh_canvas_q: Query<(&ExtractedSyltMeshCanvas, &Children)>,
    mesh_scene_q: Query<(&ExtractedSyltMeshScene, &PreparedAffine)>,
    mesh_text_q: Query<(&ExtractedSyltMeshText, &PreparedAffine)>,
    mesh_svg_q: Query<(&ExtractedSyltMeshSvg, &PreparedAffine)>,
) {
    enum RenderItem<'a> {
        Scene(&'a ExtractedSyltMeshScene),
        Text(&'a ExtractedSyltMeshText),
        Svg(&'a ExtractedSyltMeshSvg),
    }

    for (mesh_canvas, children) in mesh_canvas_q.iter() {
        let mut master_scene = vello::Scene::new();

        let gpu_image = gpu_images.get(&mesh_canvas.inner.handle).unwrap();

        let mut render_queue: Vec<(i32, vello::kurbo::Affine, RenderItem)> =
            vec![];

        for child in children.iter() {
            if let Ok((mesh_scene, affine)) = mesh_scene_q.get(child) {
                render_queue.push((
                    mesh_scene.transform.translation.z as i32,
                    **affine,
                    RenderItem::Scene(mesh_scene),
                ));
            }
            if let Ok((mesh_text, affine)) = mesh_text_q.get(child) {
                render_queue.push((
                    mesh_text.transform.translation.z as i32,
                    **affine,
                    RenderItem::Text(mesh_text),
                ));
            }
            if let Ok((mesh_svg, affine)) = mesh_svg_q.get(child) {
                render_queue.push((
                    mesh_svg.transform.translation.z as i32,
                    **affine,
                    RenderItem::Svg(mesh_svg),
                ));
            }
        }

        render_queue.sort_by(|a, b| a.0.cmp(&b.0));

        for (_, affine, render_item) in render_queue.into_iter() {
            match render_item {
                RenderItem::Scene(extracted_scene) => {
                    master_scene
                        .append(&extracted_scene.inner.inner, Some(affine));
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
                    //     // SyltSvgGlyph::Switch(glyph) => {
                    //     //     sylt_svgs.switch.get(&glyph)
                    //     // }
                    //     // SyltSvgGlyph::Xbox(glyph) => sylt_svgs.xbox.get(&glyph),
                    // };
                    //
                    // if let Some(svg_handle) = svg_collection {
                    //     let svg = svg_assets.get(svg_handle).unwrap();
                    //     extracted_svg.inner.render(
                    //         &mut master_scene,
                    //         affine,
                    //         svg,
                    //         &extracted_svg.svg_anchor,
                    //     );
                    // }
                }
            }
        }

        let params = RenderParams {
            base_color: mesh_canvas.inner.base_color,
            width: gpu_image.size.width,
            height: gpu_image.size.height,
            // TODO: from settings
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
}
