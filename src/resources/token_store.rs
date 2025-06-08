use std::sync::Arc;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct TokenStore {
    pub account_id: Option<Arc<str>>,
    pub invite_token: Option<Arc<str>>,
    pub auth_token: Option<Arc<str>>,
}
