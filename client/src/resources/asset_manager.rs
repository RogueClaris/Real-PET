use super::SoundBuffer;
use framework::prelude::*;
use std::sync::Arc;

pub trait AssetManager {
    fn local_path(&self, path: &str) -> String;
    fn binary(&self, path: &str) -> Vec<u8>;
    fn text(&self, path: &str) -> String;
    fn texture(&self, game_io: &GameIO, path: &str) -> Arc<Texture>;
    fn audio(&self, path: &str) -> SoundBuffer;

    fn new_sprite(&self, game_io: &GameIO, texture_path: &str) -> Sprite {
        let texture = self.texture(game_io, texture_path);

        Sprite::new(game_io, texture)
    }
}
