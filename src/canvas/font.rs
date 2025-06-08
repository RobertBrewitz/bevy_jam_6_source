use bevy::prelude::*;

use super::font_context::{get_global_font_context, LOCAL_FONT_CONTEXT};

pub trait SyltFontAppExt {
    fn add_sylt_font(&mut self, bytes: Vec<u8>) -> &mut Self;
}

impl SyltFontAppExt for App {
    fn add_sylt_font(&mut self, bytes: Vec<u8>) -> &mut Self {
        LOCAL_FONT_CONTEXT.with_borrow_mut(|font_context| {
            if font_context.is_none() {
                *font_context = Some(get_global_font_context().clone());
            }
            let font_context = font_context.as_mut().unwrap();
            let registered_fonts =
                font_context.collection.register_fonts(bytes.into(), None);
            let maybe_font = registered_fonts.first();
            if maybe_font.is_none() {
                warn!("Failed to register default font");
            }
        });

        self
    }
}
