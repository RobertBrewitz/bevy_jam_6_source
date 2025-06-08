use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{renderer::RenderDevice, RenderApp},
};
use svg::SyltSvgPlugin;
use text::SyltTextPlugin;
use vello::AaSupport;

use mesh_canvas::SyltMeshCanvasPlugin;
use ui_canvas::SyltUiCanvasPlugin;
use world_canvas::SyltWorldCanvasPlugin;

pub mod font;
pub mod font_context;
pub mod mesh_canvas;
pub mod svg;
pub mod text;
pub mod ui_canvas;
pub mod world_canvas;

pub struct SyltCanvasPlugin;

impl Plugin for SyltCanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SyltUiCanvasPlugin,
            SyltWorldCanvasPlugin,
            SyltMeshCanvasPlugin,
            SyltSvgPlugin,
            SyltTextPlugin,
        ));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<VelloRenderer>();
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct VelloRenderer(Arc<Mutex<vello::Renderer>>);

impl VelloRenderer {
    pub fn try_new(device: &vello::wgpu::Device) -> Result<Self, vello::Error> {
        vello::Renderer::new(
            device,
            vello::RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map(Mutex::new)
        .map(Arc::new)
        .map(VelloRenderer)
    }
}

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        match VelloRenderer::try_new(
            world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("Attempting safe-mode fallback, failed to initialize renderer: {e:}");
                match VelloRenderer::try_new(
                    world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
                ) {
                    Ok(r) => r,
                    Err(e) => panic!("Failed to start vello: {e}"),
                }
            }
        }
    }
}
