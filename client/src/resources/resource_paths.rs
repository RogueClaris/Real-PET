use framework::prelude::PlatformApp;
use std::sync::OnceLock;

static GAME_PATH: OnceLock<String> = OnceLock::new();

pub struct ResourcePaths;

impl ResourcePaths {
    pub const SERVER_CACHE_FOLDER: &str = "cache/servers/";
    pub const MOD_CACHE_FOLDER: &str = "cache/mods/";
    pub const IDENTITY_FOLDER: &str = "identity/";
    pub const VIRTUAL_PREFIX: &str = "/virtual/";
    pub const SEPARATOR: &str = "/";

    // Music
    pub const SOUND_FONT: &str = "resources/music/soundfont.sf2";
    pub const MAIN_MENU_MUSIC: &str = "resources/music/main_menu.ogg";
    pub const CUSTOMIZE_MUSIC: &str = "resources/music/customize.ogg";
    pub const BATTLE_MUSIC: &str = "resources/music/battle.ogg";

    // SFX
    pub const START_GAME_SFX: &str = "resources/sfx/start_game.ogg";
    pub const CURSOR_MOVE_SFX: &str = "resources/sfx/cursor_move.ogg";
    pub const CURSOR_SELECT_SFX: &str = "resources/sfx/cursor_select.ogg";
    pub const CURSOR_CANCEL_SFX: &str = "resources/sfx/cursor_cancel.ogg";
    pub const CURSOR_ERROR_SFX: &str = "resources/sfx/cursor_error.ogg";
    pub const MENU_CLOSE_SFX: &str = "resources/sfx/menu_close.ogg";
    pub const PAGE_TURN_SFX: &str = "resources/sfx/page_open.ogg";
    pub const TEXT_BLIP_SFX: &str = "resources/sfx/text.ogg";
    pub const CUSTOMIZE_START_SFX: &str = "resources/sfx/customize_start.ogg";
    pub const CUSTOMIZE_EMPTY_SFX: &str = "resources/sfx/customize_empty.ogg";
    pub const CUSTOMIZE_BLOCK_SFX: &str = "resources/sfx/customize_block.ogg";
    pub const CUSTOMIZE_COMPLETE_SFX: &str = "resources/sfx/customize_complete.ogg";
    pub const TRANSMISSION_SFX: &str = "resources/sfx/transmission.ogg";
    pub const WARP_SFX: &str = "resources/sfx/warp.ogg";
    pub const BATTLE_TRANSITION_SFX: &str = "resources/sfx/battle_transition.ogg";
    pub const APPEAR_SFX: &str = "resources/sfx/appear.ogg";
    pub const CARD_SELECT_OPEN_SFX: &str = "resources/sfx/card_select_open.ogg";
    pub const CARD_SELECT_CONFIRM_SFX: &str = "resources/sfx/card_select_confirm.ogg";
    pub const FORM_SELECT_OPEN_SFX: &str = "resources/sfx/page_open.ogg";
    pub const FORM_SELECT_CLOSE_SFX: &str = "resources/sfx/page_close.ogg";
    pub const TURN_GAUGE_SFX: &str = "resources/sfx/turn_gauge_full.ogg";
    pub const TIME_FREEZE_SFX: &str = "resources/sfx/time_freeze.ogg";
    pub const TILE_BREAK_SFX: &str = "resources/sfx/tile_break.ogg";
    pub const TRAP_SFX: &str = "resources/sfx/trap.ogg";
    pub const SHINE_SFX: &str = "resources/sfx/shine.ogg";
    pub const TRANSFORM_SELECT_SFX: &str = "resources/sfx/transform_select.ogg";
    pub const TRANSFORM_SFX: &str = "resources/sfx/transform.ogg";
    pub const TRANSFORM_REVERT_SFX: &str = "resources/sfx/transform_revert.ogg";
    pub const ATTACK_CHARGING_SFX: &str = "resources/sfx/attack_charging.ogg";
    pub const ATTACK_CHARGED_SFX: &str = "resources/sfx/attack_charged.ogg";
    pub const COUNTER_HIT_SFX: &str = "resources/sfx/counter_hit.ogg";
    pub const LOW_HP_SFX: &str = "resources/sfx/low_hp.ogg";
    pub const PLAYER_DELETED_SFX: &str = "resources/sfx/player_deleted.ogg";
    pub const HURT_SFX: &str = "resources/sfx/hurt.ogg";
    pub const EXPLODE_SFX: &str = "resources/sfx/explode.ogg";

    // General
    pub const BLANK: &str = "";
    pub const WHITE_PIXEL: &str = "resources/scenes/shared/white_pixel.png";
    pub const FONTS: &str = "resources/fonts/fonts_compressed.png";
    pub const FONTS_ANIMATION: &str = "resources/fonts/fonts_compressed.animation";
    pub const SUB_SCENE: &str = "resources/scenes/shared/sub_scene.png";
    pub const SUB_SCENE_ANIMATION: &str = "resources/scenes/shared/sub_scene.animation";
    pub const PAGE_ARROW: &str = "resources/scenes/shared/page_arrow.png";
    pub const PAGE_ARROW_ANIMATION: &str = "resources/scenes/shared/page_arrow.animation";
    pub const SELECT_CURSOR: &str = "resources/scenes/shared/select_cursor.png";
    pub const SELECT_CURSOR_ANIMATION: &str = "resources/scenes/shared/select_cursor.animation";
    pub const SCROLLBAR_THUMB: &str = "resources/scenes/shared/scrollbar.png";
    pub const UI_NINE_PATCHES: &str = "resources/scenes/shared/ui_nine_patches.png";
    pub const UI_NINE_PATCHES_ANIMATION: &str = "resources/scenes/shared/ui_nine_patches.animation";
    pub const ELEMENTS: &str = "resources/scenes/shared/elements.png";
    pub const CARD_ICON_MISSING: &str = "resources/scenes/shared/card_icon_missing.png";
    pub const CARD_PREVIEW_MISSING: &str = "resources/scenes/shared/card_missing.png";
    pub const REGULAR_CARD: &str = "resources/scenes/shared/regular_card.png";
    pub const REGULAR_CARD_ANIMATION: &str = "resources/scenes/shared/regular_card.animation";
    pub const FULL_CARD: &str = "resources/scenes/shared/full_card.png";
    pub const FULL_CARD_ANIMATION: &str = "resources/scenes/shared/full_card.animation";
    pub const HEALTH_FRAME: &str = "resources/scenes/shared/health_frame.png";
    pub const HEALTH_FRAME_ANIMATION: &str = "resources/scenes/shared/health_frame.animation";
    pub const UNREAD: &str = "resources/scenes/shared/unread.png";
    pub const UNREAD_ANIMATION: &str = "resources/scenes/shared/unread.animation";
    pub const INPUT_OVERLAY: &str = "resources/scenes/shared/input_overlay.png";
    pub const INPUT_OVERLAY_ANIMATION: &str = "resources/scenes/shared/input_overlay.animation";

    // Textbox
    pub const TEXTBOX_CURSOR: &str = "resources/scenes/shared/textbox_cursor.png";
    pub const TEXTBOX_CURSOR_ANIMATION: &str = "resources/scenes/shared/textbox_cursor.animation";
    pub const TEXTBOX_NEXT: &str = "resources/scenes/shared/textbox_next.png";
    pub const TEXTBOX_NEXT_ANIMATION: &str = "resources/scenes/shared/textbox_next.animation";
    pub const NAVIGATION_TEXTBOX: &str = "resources/scenes/shared/navigation_textbox.png";
    pub const NAVIGATION_TEXTBOX_ANIMATION: &str =
        "resources/scenes/shared/navigation_textbox.animation";

    // BootScene
    pub const BOOT_BG: &str = "resources/scenes/boot/bg.png";
    pub const BOOT_BG_ANIMATION: &str = "resources/scenes/boot/bg.animation";
    pub const BOOT_UI: &str = "resources/scenes/boot/ui.png";
    pub const BOOT_UI_ANIMATION: &str = "resources/scenes/boot/ui.animation";

    // MainMenuScene
    pub const MAIN_MENU_BG: &str = "resources/scenes/main_menu/bg.png";
    pub const MAIN_MENU_BG_ANIMATION: &str = "resources/scenes/main_menu/bg.animation";
    pub const MAIN_MENU_LAYOUT_ANIMATION: &str = "resources/scenes/main_menu/layout.animation";
    pub const MAIN_MENU_PARTS: &str = "resources/scenes/main_menu/menu.png";
    pub const MAIN_MENU_PARTS_ANIMATION: &str = "resources/scenes/main_menu/menu.animation";

    // ServerListScene
    pub const SERVER_LIST_STATUS: &str = "resources/scenes/server_list/status.png";
    pub const SERVER_LIST_STATUS_ANIMATION: &str = "resources/scenes/server_list/status.animation";
    pub const SERVER_LIST_LAYOUT_ANIMATION: &str = "resources/scenes/server_list/layout.animation";

    // InitialConnectScene
    pub const INITIAL_CONNECT_BG: &str = "resources/scenes/initial_connection/bg.png";
    pub const INITIAL_CONNECT_BG_ANIMATION: &str =
        "resources/scenes/initial_connection/bg.animation";

    // OverworldSceneBase
    pub const OVERWORLD_TEXTBOX: &str = "resources/scenes/overworld/textbox.png";
    pub const OVERWORLD_TEXTBOX_ANIMATION: &str = "resources/scenes/overworld/textbox.animation";
    pub const OVERWORLD_WARP: &str = "resources/scenes/overworld/warp.png";
    pub const OVERWORLD_WARP_ANIMATION: &str = "resources/scenes/overworld/warp.animation";
    pub const OVERWORLD_EMOTES: &str = "resources/scenes/overworld/emotes/emotes.png";
    pub const OVERWORLD_EMOTES_ANIMATION: &str =
        "resources/scenes/overworld/emotes/emotes.animation";
    pub const OVERWORLD_EMOTE_UI: &str = "resources/scenes/overworld/emotes/ui.png";
    pub const OVERWORLD_EMOTE_UI_ANIMATION: &str = "resources/scenes/overworld/emotes/ui.animation";
    pub const OVERWORLD_MAP_BG: &str = "resources/scenes/overworld/map/bg.png";
    pub const OVERWORLD_MAP_BG_ANIMATION: &str = "resources/scenes/overworld/map/bg.animation";
    pub const OVERWORLD_MAP_OVERLAY: &str = "resources/scenes/overworld/map/overlay.png";
    pub const OVERWORLD_MAP_OVERLAY_ARROWS: &str =
        "resources/scenes/overworld/map/overlay_arrows.png";
    pub const OVERWORLD_MAP_MARKERS: &str = "resources/scenes/overworld/map/markers.png";
    pub const OVERWORLD_MAP_MARKERS_ANIMATION: &str =
        "resources/scenes/overworld/map/markers.animation";
    pub const OVERWORLD_BBS: &str = "resources/scenes/overworld/bbs/bbs.png";
    pub const OVERWORLD_BBS_ANIMATION: &str = "resources/scenes/overworld/bbs/bbs.animation";
    pub const OVERWORLD_SHOP_BG: &str = "resources/scenes/overworld/shop/bg.png";
    pub const OVERWORLD_SHOP_BG_ANIMATION: &str = "resources/scenes/overworld/shop/bg.animation";
    pub const OVERWORLD_SHOP: &str = "resources/scenes/overworld/shop/ui.png";
    pub const OVERWORLD_SHOP_ANIMATION: &str = "resources/scenes/overworld/shop/ui.animation";
    pub const ITEMS_LAYOUT_ANIMATION: &str = "resources/scenes/overworld/items/layout.animation";
    pub const ITEM_DESCRIPTION: &str = "resources/scenes/overworld/items/item_bg.png";

    // DeckListScene
    pub const DECKS_LAYOUT_ANIMATION: &str = "resources/scenes/deck_list/layout.animation";
    pub const DECKS_ENABLED: &str = "resources/scenes/deck_list/enabled.png";
    pub const DECKS_DISABLED: &str = "resources/scenes/deck_list/disabled.png";
    pub const DECKS_FRAME: &str = "resources/scenes/deck_list/frame.png";
    pub const DECKS_CURSOR: &str = "resources/scenes/deck_list/cursor.png";
    pub const DECKS_CURSOR_ANIMATION: &str = "resources/scenes/deck_list/cursor.animation";
    pub const DECKS_EQUIPPED: &str = "resources/scenes/deck_list/equip.png";
    pub const DECKS_EQUIPPED_ANIMATION: &str = "resources/scenes/deck_list/equip.animation";

    // DeckEditorScene
    pub const DECK_UI: &str = "resources/scenes/deck_editor/ui.png";
    pub const DECK_UI_ANIMATION: &str = "resources/scenes/deck_editor/ui.animation";

    // LibraryScene
    pub const LIBRARY_LAYOUT_ANIMATION: &str = "resources/scenes/library/layout.animation";
    pub const LIBRARY_DOCK: &str = "resources/scenes/library/dock.png";
    pub const LIBRARY_DOCK_ANIMATION: &str = "resources/scenes/library/dock.animation";

    // CharacterScene
    pub const CHARACTER_PAGES: &str = "resources/scenes/character_status/pages.png";
    pub const CHARACTER_PAGES_ANIMATION: &str = "resources/scenes/character_status/pages.animation";

    // CustomizeScene
    pub const CUSTOMIZE_BG: &str = "resources/scenes/customize/bg.png";
    pub const CUSTOMIZE_BG_ANIMATION: &str = "resources/scenes/customize/bg.animation";
    pub const CUSTOMIZE_UI: &str = "resources/scenes/customize/ui.png";
    pub const CUSTOMIZE_UI_ANIMATION: &str = "resources/scenes/customize/ui.animation";
    pub const CUSTOMIZE_PREVIEW_ANIMATION: &str = "resources/scenes/customize/preview.animation";

    // CharacterSelectScene
    pub const CHARACTER_SELECT_LAYOUT_ANIMATION: &str =
        "resources/scenes/character_select/layout.animation";
    pub const CHARACTER_SELECT_CURSOR: &str = "resources/scenes/character_select/cursor.png";
    pub const CHARACTER_SELECT_CURSOR_ANIMATION: &str =
        "resources/scenes/character_select/cursor.animation";
    pub const CHARACTER_SELECT_INVALID: &str = "resources/scenes/character_select/invalid.png";
    pub const CHARACTER_SELECT_INVALID_ANIMATION: &str =
        "resources/scenes/character_select/invalid.animation";

    // KeyItemsScene
    pub const KEY_ITEMS_LAYOUT_ANIMATION: &str = "resources/scenes/key_items/layout.animation";
    pub const KEY_ITEMS_MUG: &str = "resources/scenes/key_items/mug.png";

    // BattleSelectScene
    pub const BATTLE_SELECT_UI: &str = "resources/scenes/battle_select/ui.png";
    pub const BATTLE_SELECT_UI_ANIMATION: &str = "resources/scenes/battle_select/ui.animation";

    // BattleScene
    pub const BATTLE_BG: &str = "resources/scenes/battle/bg.png";
    pub const BATTLE_BG_ANIMATION: &str = "resources/scenes/battle/bg.animation";
    pub const BATTLE_RED_TILES: &str = "resources/scenes/battle/tile_atlas_red.png";
    pub const BATTLE_BLUE_TILES: &str = "resources/scenes/battle/tile_atlas_blue.png";
    pub const BATTLE_OTHER_TILES: &str = "resources/scenes/battle/tile_atlas_other.png";
    pub const BATTLE_TILE_ANIMATION: &str = "resources/scenes/battle/tiles.animation";
    pub const BATTLE_SHADOW_SMALL: &str = "resources/scenes/battle/shadow_small.png";
    pub const BATTLE_SHADOW_BIG: &str = "resources/scenes/battle/shadow_big.png";
    pub const BATTLE_CHARGE: &str = "resources/scenes/battle/charge.png";
    pub const BATTLE_CHARGE_ANIMATION: &str = "resources/scenes/battle/charge.animation";
    pub const BATTLE_CARD_CHARGE: &str = "resources/scenes/battle/card_charge.png";
    pub const BATTLE_CARD_CHARGE_ANIMATION: &str = "resources/scenes/battle/card_charge.animation";
    pub const BATTLE_TRANSFORM_SHINE: &str = "resources/scenes/battle/transform_shine.png";
    pub const BATTLE_TRANSFORM_SHINE_ANIMATION: &str =
        "resources/scenes/battle/transform_shine.animation";
    pub const BATTLE_CARD_SELECT: &str = "resources/scenes/battle/card_select.png";
    pub const BATTLE_CARD_SELECT_ANIMATION: &str = "resources/scenes/battle/card_select.animation";
    pub const BATTLE_TURN_GAUGE: &str = "resources/scenes/battle/turn_gauge.png";
    pub const BATTLE_TURN_GAUGE_ANIMATION: &str = "resources/scenes/battle/turn_gauge.animation";
    pub const BATTLE_EXPLOSION: &str = "resources/scenes/battle/explosion.png";
    pub const BATTLE_EXPLOSION_ANIMATION: &str = "resources/scenes/battle/explosion.animation";
    pub const BATTLE_SPLASH: &str = "resources/scenes/battle/splash.png";
    pub const BATTLE_SPLASH_ANIMATION: &str = "resources/scenes/battle/splash.animation";
    pub const BATTLE_STATUSES: &str = "resources/scenes/battle/statuses.png";
    pub const BATTLE_STATUSES_ANIMATION: &str = "resources/scenes/battle/statuses.animation";
    pub const BATTLE_POOF: &str = "resources/scenes/battle/poof.png";
    pub const BATTLE_POOF_ANIMATION: &str = "resources/scenes/battle/poof.animation";
    pub const BATTLE_ALERT: &str = "resources/scenes/battle/alert.png";
    pub const BATTLE_ALERT_ANIMATION: &str = "resources/scenes/battle/alert.animation";

    // ConfigScene
    pub const CONFIG_LAYOUT_ANIMATION: &str = "resources/scenes/config/layout.animation";

    // PackagesScene
    pub const PACKAGES_LAYOUT_ANIMATION: &str = "resources/scenes/packages/layout.animation";
    pub const PACKAGES_PREVIEW: &str = "resources/scenes/packages/preview.png";
    pub const PACKAGES_PREVIEW_ANIMATION: &str = "resources/scenes/packages/preview.animation";

    // PackageScene
    pub const PACKAGE_LAYOUT_ANIMATION: &str = "resources/scenes/package/layout.animation";

    // PackageUpdatesScene
    pub const PACKAGE_UPDATES_LAYOUT_ANIMATION: &str =
        "resources/scenes/package_updates/layout.animation";

    // ResourceOrderScene
    pub const RESOURCE_ORDER_LAYOUT_ANIMATION: &str =
        "resources/scenes/resource_order/layout.animation";

    #[allow(unused_variables)]
    pub fn init_game_folder(app: &PlatformApp) {
        #[cfg(not(target_os = "android"))]
        let _ = GAME_PATH.set(ResourcePaths::clean_folder(
            &std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy(),
        ));

        #[cfg(target_os = "android")]
        let _ = GAME_PATH.set(ResourcePaths::clean_folder(
            &app.internal_data_path().unwrap().to_string_lossy(),
        ));
    }

    pub fn game_folder() -> &'static str {
        GAME_PATH.get().unwrap()
    }

    pub fn is_absolute(path_str: &str) -> bool {
        use std::path::Path;

        path_str.starts_with('/') || Path::new(&path_str).is_absolute()
    }

    pub fn absolute(path_str: &str) -> String {
        Self::game_folder().to_string() + &Self::clean(path_str)
    }

    pub fn clean(path_str: &str) -> String {
        path_clean::clean(path_str)
            .to_str()
            .unwrap_or_default()
            .replace('\\', Self::SEPARATOR)
    }

    pub fn clean_folder(path_str: &str) -> String {
        Self::clean(path_str) + Self::SEPARATOR
    }

    pub fn parent(path_str: &str) -> Option<&str> {
        let last_index = path_str.len().max(1) - 1;
        let slash_index = path_str[..last_index].rfind('/')?;

        Some(&path_str[..slash_index + 1])
    }

    pub fn shorten(path_str: &str) -> String {
        let game_path = Self::game_folder();

        path_str
            .strip_prefix(game_path)
            .unwrap_or(path_str)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean() {
        assert_eq!(ResourcePaths::clean("./.\\test"), String::from("test"));
        assert_eq!(ResourcePaths::clean("../test"), String::from("../test"));
        assert_eq!(ResourcePaths::clean("..\\test"), String::from("../test"));
    }

    #[test]
    fn parent() {
        assert_eq!(ResourcePaths::parent("a/b/c"), Some("a/b/"));
        assert_eq!(ResourcePaths::parent("a/b/"), Some("a/"));
        assert_eq!(ResourcePaths::parent("a/"), None);
        assert_eq!(ResourcePaths::parent("a"), None);
        assert_eq!(ResourcePaths::parent(""), None);
    }
}
