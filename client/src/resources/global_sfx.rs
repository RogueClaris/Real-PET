use super::{ResourcePaths, SoundBuffer};
use field_count::FieldCount;

#[derive(Default, FieldCount)]
pub struct GlobalSfx {
    pub start_game: SoundBuffer,
    pub cursor_move: SoundBuffer,
    pub cursor_select: SoundBuffer,
    pub cursor_cancel: SoundBuffer,
    pub cursor_error: SoundBuffer,
    pub menu_close: SoundBuffer,
    pub page_turn: SoundBuffer,
    pub text_blip: SoundBuffer,
    pub customize_start: SoundBuffer,
    pub customize_empty: SoundBuffer,
    pub customize_block: SoundBuffer,
    pub customize_complete: SoundBuffer,
    pub transmission: SoundBuffer,
    pub warp: SoundBuffer,
    pub battle_transition: SoundBuffer,
    pub appear: SoundBuffer,
    pub card_select_open: SoundBuffer,
    pub card_select_confirm: SoundBuffer,
    pub form_select_open: SoundBuffer,
    pub form_select_close: SoundBuffer,
    pub turn_gauge: SoundBuffer,
    pub time_freeze: SoundBuffer,
    pub tile_break: SoundBuffer,
    pub trap: SoundBuffer,
    pub shine: SoundBuffer,
    pub transform_select: SoundBuffer,
    pub transform: SoundBuffer,
    pub transform_revert: SoundBuffer,
    pub attack_charging: SoundBuffer,
    pub attack_charged: SoundBuffer,
    pub counter_hit: SoundBuffer,
    pub low_hp: SoundBuffer,
    pub player_deleted: SoundBuffer,
    pub hurt: SoundBuffer,
    pub explode: SoundBuffer,
}

impl GlobalSfx {
    pub fn load_with(mut load: impl FnMut(&str) -> SoundBuffer) -> Self {
        Self {
            start_game: load(ResourcePaths::START_GAME_SFX),
            cursor_move: load(ResourcePaths::CURSOR_MOVE_SFX),
            cursor_select: load(ResourcePaths::CURSOR_SELECT_SFX),
            cursor_cancel: load(ResourcePaths::CURSOR_CANCEL_SFX),
            cursor_error: load(ResourcePaths::CURSOR_ERROR_SFX),
            menu_close: load(ResourcePaths::MENU_CLOSE_SFX),
            page_turn: load(ResourcePaths::PAGE_TURN_SFX),
            text_blip: load(ResourcePaths::TEXT_BLIP_SFX),
            customize_start: load(ResourcePaths::CUSTOMIZE_START_SFX),
            customize_empty: load(ResourcePaths::CUSTOMIZE_EMPTY_SFX),
            customize_block: load(ResourcePaths::CUSTOMIZE_BLOCK_SFX),
            customize_complete: load(ResourcePaths::CUSTOMIZE_COMPLETE_SFX),
            transmission: load(ResourcePaths::TRANSMISSION_SFX),
            warp: load(ResourcePaths::WARP_SFX),
            battle_transition: load(ResourcePaths::BATTLE_TRANSITION_SFX),
            appear: load(ResourcePaths::APPEAR_SFX),
            card_select_open: load(ResourcePaths::CARD_SELECT_OPEN_SFX),
            card_select_confirm: load(ResourcePaths::CARD_SELECT_CONFIRM_SFX),
            form_select_open: load(ResourcePaths::FORM_SELECT_OPEN_SFX),
            form_select_close: load(ResourcePaths::FORM_SELECT_CLOSE_SFX),
            turn_gauge: load(ResourcePaths::TURN_GAUGE_SFX),
            time_freeze: load(ResourcePaths::TIME_FREEZE_SFX),
            tile_break: load(ResourcePaths::TILE_BREAK_SFX),
            trap: load(ResourcePaths::TRAP_SFX),
            shine: load(ResourcePaths::SHINE_SFX),
            transform_select: load(ResourcePaths::TRANSFORM_SELECT_SFX),
            transform: load(ResourcePaths::TRANSFORM_SFX),
            transform_revert: load(ResourcePaths::TRANSFORM_REVERT_SFX),
            attack_charging: load(ResourcePaths::ATTACK_CHARGING_SFX),
            attack_charged: load(ResourcePaths::ATTACK_CHARGED_SFX),
            counter_hit: load(ResourcePaths::COUNTER_HIT_SFX),
            low_hp: load(ResourcePaths::LOW_HP_SFX),
            player_deleted: load(ResourcePaths::PLAYER_DELETED_SFX),
            hurt: load(ResourcePaths::HURT_SFX),
            explode: load(ResourcePaths::EXPLODE_SFX),
        }
    }

    pub fn total() -> usize {
        Self::field_count()
    }
}
