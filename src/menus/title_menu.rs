use bevy::{prelude::*, render::view::RenderLayers, ui::ContentSize};

use crate::{
    canvas::{
        text::{SyltText, SyltTextStyle},
        ui_canvas::SyltUiText,
    },
    routes::SyltRouterState,
    ui::{
        cardinal_navigation::{
            CardinalCrosshairExt, SyltCardinalFocusedEvent,
            SyltCardinalNavigation,
        },
        components::button::{
            SyltButtonExt, SyltButtonFocused, SyltButtonNavigationExt,
            SyltButtonPressed,
        },
        constants::{SU4, SU8},
        escape::SyltEscape,
        layouts::layout_wrapper,
    },
};

use super::SyltMenuState;

pub struct TitleMenuPlugin;

impl Plugin for TitleMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLeftMenuItem>();
        app.init_resource::<LastRightMenuItem>();

        app.add_systems(OnEnter(SyltRouterState::Title), spawn_title_menu);
    }
}

#[derive(Component)]
struct LeftItem;

#[derive(Resource, Default, Deref, DerefMut)]
struct LastLeftMenuItem(pub Option<Entity>);

fn last_left_menu_item_observer<E>(
) -> impl Fn(Trigger<E>, Query<&mut SyltCardinalNavigation, With<RightItem>>) {
    move |ev, mut right_items| {
        for mut item in right_items.iter_mut() {
            item.west = Some(ev.target());
        }
    }
}

#[derive(Component)]
struct RightItem;

#[derive(Resource, Default, Deref, DerefMut)]
struct LastRightMenuItem(pub Option<Entity>);

fn last_right_menu_item_observer<E>(
) -> impl Fn(Trigger<E>, Query<&mut SyltCardinalNavigation, With<LeftItem>>) {
    move |ev, mut left_items| {
        for mut item in left_items.iter_mut() {
            item.east = Some(ev.target());
        }
    }
}

fn spawn_title_menu(
    mut cmd: Commands,
    mut event_writer: EventWriter<SyltCardinalFocusedEvent>,
    last_left_menu_item_res: Res<LastLeftMenuItem>,
) {
    cmd.spawn_cardinal_crosshair(StateScoped(SyltRouterState::Title));

    cmd.spawn((
        StateScoped(SyltRouterState::Title),
        Name::new("Splash Container"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(32.0),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            RenderLayers::layer(1),
            SyltUiText,
            SyltTextStyle {
                font_size: 75.,
                ..default()
            },
            SyltText {
                content: "Hrodban's".to_string(),
                ..default()
            },
            Node::default(),
            ContentSize::default(),
        ));

        parent.spawn((
            RenderLayers::layer(1),
            SyltUiText,
            SyltTextStyle {
                font_size: 75.,
                ..default()
            },
            SyltText {
                content: env!("CARGO_PKG_NAME").to_string(),
                ..default()
            },
            Node::default(),
            ContentSize::default(),
        ));
    });

    let wrapper = cmd
        .spawn((StateScoped(SyltMenuState::Title), layout_wrapper()))
        .id();

    let external_links_container = cmd
        .spawn((Node {
            width: Val::Percent(100.),
            max_width: Val::Px(640.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(SU4),
            padding: UiRect::all(Val::Px(SU8)),
            ..default()
        },))
        .id();

    let youtube_button = cmd
        .spawn_sylt_button("youtube", RightItem)
        .observe(last_right_menu_item_observer::<SyltButtonFocused>())
        .observe(move |_: Trigger<Pointer<Released>>| {
            if let Ok(youtube_url) = dotenv::var("YOUTUBE_URL") {
                #[cfg(not(target_arch = "wasm32"))]
                open::that(youtube_url).unwrap();

                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    window.open_with_url_and_target(&youtube_url, "_blank");
                }
            }
        })
        .observe(move |_: Trigger<SyltButtonPressed>| {
            if let Ok(youtube_url) = dotenv::var("YOUTUBE_URL") {
                #[cfg(not(target_arch = "wasm32"))]
                open::that(youtube_url).unwrap();

                #[cfg(target_arch = "wasm32")]
                if let Some(window) = web_sys::window() {
                    window.open_with_url_and_target(&youtube_url, "_blank");
                }
            }
        })
        .id();

    // let discord_button = cmd
    //     .spawn_sylt_button("discord", RightItem)
    //     .observe(last_right_menu_item_observer::<SyltButtonFocused>())
    //     .observe(move |_: Trigger<Pointer<Released>>| {
    //         if let Ok(discord_url) = dotenv::var("DISCORD_URL") {
    //             #[cfg(not(target_arch = "wasm32"))]
    //             open::that(discord_url).unwrap();
    //
    //             #[cfg(target_arch = "wasm32")]
    //             if let Some(window) = web_sys::window() {
    //                 window.open_with_url_and_target(&discord_url, "_blank");
    //             }
    //         }
    //     })
    //     .observe(move |_: Trigger<SyltButtonPressed>| {
    //         if let Ok(discord_url) = dotenv::var("DISCORD_URL") {
    //             #[cfg(not(target_arch = "wasm32"))]
    //             open::that(discord_url).unwrap();
    //
    //             #[cfg(target_arch = "wasm32")]
    //             if let Some(window) = web_sys::window() {
    //                 window.open_with_url_and_target(&discord_url, "_blank");
    //             }
    //         }
    //     })
    //     .id();

    let menu_container = cmd
        .spawn((Node {
            width: Val::Percent(100.),
            max_width: Val::Px(640.),
            height: Val::Percent(100.),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(SU4),
            padding: UiRect::all(Val::Px(SU8)),
            ..default()
        },))
        .id();

    // TODO: Disable if no save games
    // let continue_button = cmd
    //     .spawn_sylt_button("continue", LeftItem)
    //     .navigate_on_click(SyltRouterState::Continue)
    //     .observe(last_left_menu_item_observer::<SyltButtonFocused>())
    //     .id();

    let new_game_button = cmd
        .spawn_sylt_button("new game", LeftItem)
        .navigate_on_click(SyltRouterState::Game)
        .observe(last_left_menu_item_observer::<SyltButtonFocused>())
        .id();

    // let load_game_button = cmd
    //     .spawn_sylt_button("load game", LeftItem)
    //     .navigate_on_click(SyltRouterState::LoadGame)
    //     .observe(last_left_menu_item_observer::<SyltButtonFocused>())
    //     .id();

    // let play_online_button = cmd
    //     .spawn_sylt_button("play online", LeftItem)
    //     .navigate_on_click(SyltRouterState::RedeemInviteToken)
    //     .observe(last_left_menu_item_observer::<SyltButtonFocused>())
    //     .id();

    let settings_button = cmd
        .spawn_sylt_button("settings", LeftItem)
        .navigate_on_click(SyltRouterState::Settings)
        .observe(last_left_menu_item_observer::<SyltButtonFocused>())
        .id();

    // let exit_button = cmd
    //     .spawn_sylt_button("exit", LeftItem)
    //     .navigate_on_click(SyltRouterState::Exit)
    //     .observe(last_left_menu_item_observer::<SyltButtonFocused>())
    //     .id();

    // cmd.entity(continue_button).insert(SyltCardinalNavigation {
    //     north: Some(exit_button),
    //     south: Some(new_game_button),
    //     east: Some(youtube_button),
    //     ..default()
    // });

    cmd.entity(new_game_button).insert(SyltCardinalNavigation {
        north: Some(settings_button),
        south: Some(settings_button),
        east: Some(youtube_button),
        ..default()
    });

    // cmd.entity(load_game_button).insert(SyltCardinalNavigation {
    //     north: Some(new_game_button),
    //     south: Some(play_online_button),
    //     east: Some(youtube_button),
    //     ..default()
    // });
    //
    // cmd.entity(play_online_button)
    //     .insert(SyltCardinalNavigation {
    //         north: Some(load_game_button),
    //         south: Some(settings_button),
    //         east: Some(youtube_button),
    //         ..default()
    //     });

    cmd.entity(settings_button).insert(SyltCardinalNavigation {
        north: Some(new_game_button),
        south: Some(new_game_button),
        east: Some(youtube_button),
        ..default()
    });

    // cmd.entity(exit_button).insert((
    //     SyltCardinalNavigation {
    //         north: Some(settings_button),
    //         south: Some(new_game_button),
    //         east: Some(youtube_button),
    //         ..default()
    //     },
    //     SyltEscape,
    // ));

    cmd.entity(youtube_button).insert(SyltCardinalNavigation {
        // north: Some(discord_button),
        // south: Some(discord_button),
        west: Some(last_left_menu_item_res.0.unwrap_or(new_game_button)),
        ..default()
    });

    // cmd.entity(discord_button).insert(SyltCardinalNavigation {
    //     north: Some(youtube_button),
    //     south: Some(youtube_button),
    //     west: Some(last_left_menu_item_res.0.unwrap_or(new_game_button)),
    //     ..default()
    // });

    let menu_container_id = cmd
        .entity(menu_container)
        .add_children(&[new_game_button, settings_button])
        .id();

    let external_id = cmd
        .entity(external_links_container)
        .add_children(&[youtube_button])
        .id();

    cmd.entity(wrapper)
        .add_children(&[menu_container_id, external_id]);

    // TODO: Focus continue button if save games exist
    event_writer.write(SyltCardinalFocusedEvent(Some(new_game_button)));
}
