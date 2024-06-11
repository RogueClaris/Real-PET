use crate::bindable::{AuxVariable, MathExpr};
use crate::lua_api::BattleVmManager;
use crate::packages::{PackageInfo, PackageNamespace};
use crate::render::ui::GlyphAtlas;
use crate::render::{Animator, SpriteColorQueue};
use crate::resources::{AssetManager, Globals, ResourcePaths};
use crate::scenes::BattleEvent;
use crate::{CardRecipes, RESOLUTION_F};
use framework::math::{Rect, Vec2};
use framework::prelude::{Color, GameIO, Sprite, Texture};
use packets::structures::PackageCategory;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use super::{BattleSimulation, PlayerSetup, StatusRegistry, TileState};

/// Resources that are shared between battle snapshots
pub struct SharedBattleResources {
    pub vm_manager: BattleVmManager,
    pub status_registry: StatusRegistry,
    pub statuses_texture: Arc<Texture>,
    pub statuses_animator: RefCell<Animator>,
    pub recipe_animator: Animator,
    pub recipes: CardRecipes,
    pub alert_animator: RefCell<Animator>,
    pub math_expressions:
        RefCell<HashMap<String, rollback_mlua::Result<MathExpr<f32, AuxVariable>>>>,
    pub glyph_atlases: RefCell<HashMap<(Cow<'static, str>, Cow<'static, str>), Arc<GlyphAtlas>>>,
    pub battle_fade_color: Cell<Color>,
    pub ui_fade_color: Cell<Color>,
    pub fade_sprite: Sprite,
    pub event_sender: flume::Sender<BattleEvent>,
    pub event_receiver: flume::Receiver<BattleEvent>,
}

impl SharedBattleResources {
    pub fn new(
        game_io: &GameIO,
        simulation: &mut BattleSimulation,
        player_setups: &[PlayerSetup],
        dependencies: &[(&PackageInfo, PackageNamespace)],
    ) -> Self {
        let assets = &game_io.resource::<Globals>().unwrap().assets;

        let (event_sender, event_receiver) = flume::unbounded();

        let fade_sprite_texture = assets.texture(game_io, ResourcePaths::WHITE_PIXEL);

        let mut fade_sprite = Sprite::new(game_io, fade_sprite_texture);
        fade_sprite.set_bounds(Rect::from_corners(Vec2::ZERO, RESOLUTION_F));

        let mut resources = Self {
            vm_manager: BattleVmManager::new(),
            status_registry: StatusRegistry::new(),
            statuses_texture: assets.texture(game_io, ResourcePaths::BATTLE_STATUSES),
            statuses_animator: RefCell::new(Animator::load_new(
                assets,
                ResourcePaths::BATTLE_STATUSES_ANIMATION,
            )),
            recipe_animator: Animator::load_new(assets, ResourcePaths::BATTLE_RECIPE_ANIMATION)
                .with_state("DEFAULT"),
            recipes: CardRecipes::default(),
            alert_animator: RefCell::new(
                Animator::load_new(assets, ResourcePaths::BATTLE_ALERT_ANIMATION).with_state("UI"),
            ),
            math_expressions: Default::default(),
            glyph_atlases: Default::default(),
            battle_fade_color: Default::default(),
            ui_fade_color: Default::default(),
            fade_sprite,
            event_sender,
            event_receiver,
        };

        resources.init(game_io, simulation, player_setups, dependencies);

        resources
    }

    fn init(
        &mut self,
        game_io: &GameIO,
        simulation: &mut BattleSimulation,
        player_setups: &[PlayerSetup],
        dependencies: &[(&PackageInfo, PackageNamespace)],
    ) {
        let globals = game_io.resource::<Globals>().unwrap();

        // load recipes
        for setup in player_setups {
            let ns = setup.namespace();

            for output in &setup.recipes {
                if let Some(package) = globals.card_packages.package_or_fallback(ns, output) {
                    self.recipes.load_from_package(ns, package)
                }
            }
        }

        // load tile states + statuses first
        BattleVmManager::init(
            game_io,
            self,
            simulation,
            dependencies.iter().filter(|(p, _)| {
                matches!(
                    p.category,
                    PackageCategory::TileState | PackageCategory::Status
                )
            }),
        );

        // register tile states
        TileState::complete_registry(game_io, simulation, self, dependencies);

        // register statuses
        self.status_registry
            .init(game_io, &self.vm_manager, dependencies);

        // load remaining packages
        BattleVmManager::init(
            game_io,
            self,
            simulation,
            dependencies.iter().filter(|(p, _)| {
                !matches!(
                    p.category,
                    PackageCategory::TileState | PackageCategory::Status
                )
            }),
        );
    }

    pub fn parse_math_expr(
        &self,
        source: String,
    ) -> rollback_mlua::Result<MathExpr<f32, AuxVariable>> {
        let mut expressions = self.math_expressions.borrow_mut();

        expressions
            .entry(source)
            .or_insert_with_key(|source| {
                MathExpr::parse(source)
                    .map_err(|err| rollback_mlua::Error::RuntimeError(err.to_string()))
            })
            .clone()
    }

    pub fn draw_fade_sprite(&mut self, sprite_queue: &mut SpriteColorQueue, color: Color) {
        if color != Color::TRANSPARENT {
            self.fade_sprite.set_color(color);
            sprite_queue.draw_sprite(&self.fade_sprite);
        }
    }
}
