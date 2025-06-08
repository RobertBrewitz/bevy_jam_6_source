use bevy::prelude::*;

pub mod cameras;
pub mod canvas;
pub mod game;
pub mod i18n;
pub mod menus;
pub mod resources;
pub mod routes;
pub mod settings;
pub mod signals;
pub mod sounds;
pub mod storage;
pub mod ui;
pub mod vectors;

use canvas::font::SyltFontAppExt;

pub fn main() {
    dotenv::dotenv().ok();

    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes_override: Some(true),
                #[cfg(target_arch = "wasm32")]
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!(
                        "{} v{}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    ),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..Default::default()
            }),
    );

    // Fonts
    let roboto_mono_bytes = include_bytes!(
        "../assets/fonts/roboto-mono/RobotoMono-VariableFont_wght.ttf"
    );

    app.add_sylt_font(roboto_mono_bytes.to_vec());

    // Core plugins
    app.add_plugins((
        cameras::SyltCameraPlugin,
        canvas::SyltCanvasPlugin,
        game::SyltGamePlugin,
        i18n::SyltI18nPlugin,
        resources::SyltResourcePlugin,
        routes::SyltRoutesPlugin,
        settings::SyltSettingsPlugin,
        signals::SyltSignalsPlugin,
        ui::SyltUiPlugin,
        menus::SyltMenusPlugin,
    ));

    // IO plugins
    app.add_plugins((
        sounds::SyltSoundsPlugin,
        storage::SyltStorageWrapperPlugin,
    ));

    app.run();
}
