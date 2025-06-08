use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    platform::collections::HashMap,
    prelude::*,
};
use thiserror::Error;

use crate::{
    canvas::text::SyltText, settings::SyltSettings,
    ui::system_set::SyltUiSystem,
};

pub struct SyltI18nPlugin;

impl Plugin for SyltI18nPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SyltI18nText>()
            .init_asset::<I18nFile>()
            .init_asset_loader::<I18nAssetLoader>()
            .init_resource::<I18nLocale>()
            .init_resource::<I18nData>()
            .add_systems(
                Update,
                (
                    update_i18n_data_on_locale_change,
                    set_i18n_data_on_load,
                    update_sylt_text_i18n_values,
                )
                    .in_set(SyltUiSystem::Content),
            )
            .add_systems(Startup, load_i18n_assets);
    }
}

#[derive(
    Clone, Debug, Reflect, PartialEq, serde::Deserialize, serde::Serialize, Copy,
)]
pub enum SyltLocale {
    #[serde(alias = "en")]
    English,
    #[serde(alias = "pl")]
    Polish,
    #[serde(alias = "sv")]
    Swedish,
    #[serde(alias = "es")]
    Spanish,
}

impl From<SyltLocale> for &str {
    fn from(value: SyltLocale) -> Self {
        match value {
            SyltLocale::English => "en",
            SyltLocale::Polish => "pl",
            SyltLocale::Swedish => "sv",
            SyltLocale::Spanish => "es",
        }
    }
}

impl From<&str> for SyltLocale {
    fn from(value: &str) -> Self {
        match value {
            "en" => Self::English,
            "pl" => Self::Polish,
            "sv" => Self::Swedish,
            "es" => Self::Spanish,
            _ => Self::English,
        }
    }
}

/// Use [`SyltI18nText::from_key`] to create a text that will be translated
///
/// Use [`SyltI18nText::add_template_key`] to add a key to the output for interpolation.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(SyltText)]
pub struct SyltI18nText {
    locale: SyltLocale,
    key: String,
    v: usize,
    template_keys: Vec<String>,
}

impl Default for SyltI18nText {
    fn default() -> Self {
        Self {
            locale: SyltLocale::English,
            key: String::new(),
            v: 0,
            template_keys: Vec::new(),
        }
    }
}

impl SyltI18nText {
    pub fn from_key(key: &str) -> Self {
        Self {
            key: key.to_string(),
            locale: SyltLocale::English,
            v: 0,
            template_keys: Vec::new(),
        }
    }

    pub fn update_key(&mut self, key: &str) {
        self.key = key.to_string();
    }

    pub fn add_template_key(mut self, template_key: impl Into<String>) -> Self {
        self.template_keys.push(template_key.into());
        self
    }
}

fn update_sylt_text_i18n_values(
    i18n_data: Res<I18nData>,
    mut i18n_text_q: Query<
        (&mut SyltText, &SyltI18nText),
        Changed<SyltI18nText>,
    >,
) {
    for (mut ui_text, i18n_text) in i18n_text_q.iter_mut() {
        ui_text.content = i18n_data.tr(&i18n_text.key);

        for template_key in i18n_text.template_keys.iter() {
            let content = i18n_data.tr(template_key);
            ui_text.content = ui_text.content.replacen("{}", &content, 1);
        }
    }
}

fn load_i18n_assets(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.insert_resource(I18nAssetStore {
        en: asset_server.load("i18n/en.yaml"),
        pl: asset_server.load("i18n/pl.yaml"),
        sv: asset_server.load("i18n/sv.yaml"),
        es: asset_server.load("i18n/es.yaml"),
    });
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct I18nLocale {
    pub locale: SyltLocale,
}

#[allow(clippy::too_many_arguments)]
fn set_i18n_data_on_load(
    mut asset_loaded_events: EventReader<AssetEvent<I18nFile>>,
    i18n_asset_store: Res<I18nAssetStore>,
    i18n_locale: Res<I18nLocale>,
    i18n_files: Res<Assets<I18nFile>>,
    mut i18n_data: ResMut<I18nData>,
    mut i18n_ui_texts: Query<&mut SyltI18nText>,
) {
    for event in asset_loaded_events.read() {
        match event {
            AssetEvent::LoadedWithDependencies { id } => {
                if let Some(i18n) = i18n_asset_store.get(&i18n_locale.locale) {
                    if *id == i18n.id() {
                        if let Some(asset) = i18n_files.get(i18n) {
                            if asset.locale == i18n_locale.locale {
                                if let Some(file) = i18n_files.get(i18n) {
                                    i18n_data.data = file.data.clone();
                                }
                            }
                        }
                    }
                }
            }
            AssetEvent::Modified { id } => {
                if let Some(i18n) = i18n_asset_store.get(&i18n_locale.locale) {
                    if *id == i18n.id() {
                        if let Some(asset) = i18n_files.get(i18n) {
                            if asset.locale == i18n_locale.locale {
                                if let Some(file) = i18n_files.get(i18n) {
                                    i18n_data.data = file.data.clone();

                                    for mut i18n_text in
                                        i18n_ui_texts.iter_mut()
                                    {
                                        i18n_text.v += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update_i18n_data_on_locale_change(
    i18n_asset_store: Res<I18nAssetStore>,
    mut i18n_locale: ResMut<I18nLocale>,
    mut i18n_data: ResMut<I18nData>,
    i18n_files: Res<Assets<I18nFile>>,
    mut i18n_texts: Query<&mut SyltI18nText>,
    settings: Res<SyltSettings>,
) {
    if i18n_locale.locale == settings.locale {
        return;
    }

    i18n_locale.locale = settings.locale;

    if let Some(i18n) = i18n_asset_store.get(&settings.locale) {
        if let Some(asset) = i18n_files.get(i18n) {
            i18n_data.data = asset.data.clone();
        }
    }

    for mut i18n_text in i18n_texts.iter_mut() {
        i18n_text.locale = settings.locale;
    }
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct I18nData {
    pub data: HashMap<String, String>,
}

impl I18nData {
    pub fn tr(&self, key_ref: &str) -> String {
        if let Some(res) = self.data.get(&key_ref.to_string()) {
            res.clone()
        } else {
            format!("%{key_ref}%")
        }
    }
}

impl Default for I18nLocale {
    fn default() -> Self {
        Self {
            locale: SyltLocale::English,
        }
    }
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct I18nAssetStore {
    pub en: Handle<I18nFile>,
    pub pl: Handle<I18nFile>,
    pub sv: Handle<I18nFile>,
    pub es: Handle<I18nFile>,
}

impl I18nAssetStore {
    pub fn get(&self, locale: &SyltLocale) -> Option<&Handle<I18nFile>> {
        match locale {
            SyltLocale::English => Some(&self.en),
            SyltLocale::Polish => Some(&self.pl),
            SyltLocale::Swedish => Some(&self.sv),
            SyltLocale::Spanish => Some(&self.es),
        }
    }
}

#[derive(Debug, Asset, TypePath, serde::Deserialize, serde::Serialize)]
pub struct I18nFile {
    pub data: HashMap<String, String>,
    pub locale: SyltLocale,
}

#[derive(Default)]
pub struct I18nAssetLoader;

impl AssetLoader for I18nAssetLoader {
    type Asset = I18nFile;
    type Settings = ();
    type Error = I18nAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let asset = serde_yaml::from_slice::<Self::Asset>(&bytes)?;
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        &["yaml", "yml"]
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum I18nAssetLoaderError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
