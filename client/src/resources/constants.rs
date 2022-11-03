use super::Input;
use crate::render::FrameTime;
use framework::prelude::{Color, UVec2, Vec2};

pub const INPUT_BUFFER_LIMIT: usize = 20;

pub const RESOLUTION_F: Vec2 = Vec2::new(240.0, 160.0);
pub const DEFAULT_SCALE: f32 = 2.0;
pub const TRUE_RESOLUTION: UVec2 = UVec2::new(
    (RESOLUTION_F.x * DEFAULT_SCALE) as u32,
    (RESOLUTION_F.y * DEFAULT_SCALE) as u32,
);
pub const MAX_CARDS: usize = 30;
pub const BATTLE_INPUTS: [Input; 11] = [
    Input::Up,
    Input::Down,
    Input::Left,
    Input::Right,
    Input::Shoot,
    Input::UseCard,
    Input::Special,
    Input::ShoulderL,
    Input::ShoulderR,
    Input::Confirm,
    Input::Cancel,
];

pub const BATTLE_UI_MARGIN: f32 = 2.0;

// tile states
pub const POISON_INTERVAL: FrameTime = 7;
pub const GRASS_HEAL_INTERVAL: FrameTime = 20;
pub const GRASS_SLOWED_HEAL_INTERVAL: FrameTime = 180;
pub const CONVEYOR_MOVEMENT_DELAY: FrameTime = 8;

// text colors
pub const TEXT_TRANSPARENT_SHADOW_COLOR: Color = Color::new(0.4, 0.4, 0.4, 0.3);
pub const TEXT_DARK_SHADOW_COLOR: Color = Color::new(0.32, 0.388, 0.45, 1.0);
pub const CONTEXT_TEXT_SHADOW_COLOR: Color = Color::new(0.06, 0.31, 0.41, 1.0);