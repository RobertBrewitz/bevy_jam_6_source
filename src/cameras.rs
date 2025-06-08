use bevy::{
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        view::RenderLayers,
    },
};

pub struct SyltCameraPlugin;

impl Plugin for SyltCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_ui_camera, setup_world_camera, setup_3d_camera),
        );
        app.add_plugins(ExtractComponentPlugin::<SyltUiCamera>::default());
        app.add_plugins(ExtractComponentPlugin::<SyltWorldCamera>::default());
        app.add_plugins(ExtractComponentPlugin::<Sylt3dCamera>::default());
    }
}

#[derive(Component, Clone, ExtractComponent)]
pub struct SyltUiCamera;

fn setup_ui_camera(mut cmd: Commands) {
    cmd.spawn((
        Name::new("SyltUiCamera"),
        SyltUiCamera,
        IsDefaultUiCamera,
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::None,
            order: 2,
            ..Default::default()
        },
        Msaa::Off,
        Transform::from_xyz(0., 0., 1000.),
        Projection::Orthographic(OrthographicProjection::default_2d()),
        RenderLayers::layer(1),
    ));
}

#[derive(Component, Clone, ExtractComponent)]
pub struct SyltWorldCamera;

fn setup_world_camera(mut cmd: Commands) {
    cmd.spawn((
        Name::new("SyltWorldCamera"),
        Camera2d,
        SyltWorldCamera,
        Camera {
            clear_color: ClearColorConfig::None,
            order: 1,
            ..Default::default()
        },
        Msaa::Off,
        Transform::from_xyz(0., 0., 1000.),
        Projection::Orthographic(OrthographicProjection::default_2d()),
        RenderLayers::layer(0),
    ));
}

#[derive(Component, Clone, ExtractComponent)]
pub struct Sylt3dCamera;

fn setup_3d_camera(mut cmd: Commands) {
    cmd.spawn((
        Name::new("Sytl3dCamera"),
        Sylt3dCamera,
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::hsla(
                255.0, 0.55, 0.25, 1.0,
            )),
            order: 0,
            ..Default::default()
        },
        Msaa::Off,
        Transform::from_xyz(0., 5., 1.).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection::default()),
        RenderLayers::layer(0),
    ));

    cmd.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
