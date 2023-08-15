use super::{CardClass, Element, HitFlag, HitFlags};
use crate::battle::StatusRegistry;
use crate::packages::PackageId;
use crate::render::ui::{FontStyle, TextStyle};
use crate::render::SpriteColorQueue;
use framework::prelude::{Color, GameIO, Vec2};

#[derive(Clone, PartialEq, Eq)]
pub struct CardProperties<H = HitFlags> {
    pub package_id: PackageId,
    pub code: String,
    pub short_name: String,
    pub damage: i32,
    pub boosted_damage: i32,
    pub element: Element,
    pub secondary_element: Element,
    pub card_class: CardClass,
    pub hit_flags: H,
    pub can_boost: bool,
    pub counterable: bool,
    pub time_freeze: bool,
    pub skip_time_freeze_intro: bool,
    pub meta_classes: Vec<String>,
}

impl<H: Default> Default for CardProperties<H> {
    fn default() -> Self {
        Self {
            package_id: PackageId::new_blank(),
            code: String::new(),
            short_name: String::from("?????"),
            damage: 0,
            boosted_damage: 0,
            element: Element::None,
            secondary_element: Element::None,
            time_freeze: false,
            card_class: CardClass::Standard,
            hit_flags: Default::default(),
            can_boost: true,
            counterable: true,
            skip_time_freeze_intro: false,
            meta_classes: Vec::new(),
        }
    }
}

impl CardProperties {
    pub fn draw_summary(
        &self,
        game_io: &GameIO,
        sprite_queue: &mut SpriteColorQueue,
        position: Vec2,
        center: bool,
    ) {
        let mut text_style = TextStyle::new(game_io, FontStyle::Thick);
        text_style.monospace = true;
        text_style.bounds.set_position(position);

        let name_text = &self.short_name;
        let damage_text = if self.damage == 0 {
            String::new()
        } else if self.boosted_damage == 0 {
            format!("{}", self.damage)
        } else {
            // negative sign will be provided from negative boosted_damage
            let sign_str = if self.boosted_damage > 0 { "+" } else { "" };

            format!(
                "{}{}{}",
                self.damage - self.boosted_damage,
                sign_str,
                self.boosted_damage
            )
        };

        // measure text
        let name_width = text_style.measure(name_text).size.x;

        if center {
            text_style.font_style = FontStyle::GradientOrange;
            let damage_width = text_style.measure(&damage_text).size.x;
            let text_width = name_width + text_style.letter_spacing + damage_width;

            text_style.bounds.x -= text_width * 0.5;
        }

        // draw name
        text_style.shadow_color = Color::BLACK;
        text_style.font_style = FontStyle::Thick;
        text_style.draw(game_io, sprite_queue, name_text);

        // draw damage
        text_style.shadow_color = Color::TRANSPARENT;
        text_style.font_style = FontStyle::GradientOrange;
        text_style.bounds.x += name_width + text_style.letter_spacing;
        text_style.draw(game_io, sprite_queue, &damage_text);
    }
}

impl CardProperties<Vec<String>> {
    pub fn to_bindable(&self, registry: &StatusRegistry) -> CardProperties<HitFlags> {
        CardProperties::<HitFlags> {
            package_id: self.package_id.clone(),
            code: self.code.clone(),
            short_name: self.short_name.clone(),
            damage: self.damage,
            boosted_damage: self.boosted_damage,
            element: self.element,
            secondary_element: self.secondary_element,
            card_class: self.card_class,
            hit_flags: self
                .hit_flags
                .iter()
                .map(|flag| HitFlag::from_str(registry, flag))
                .fold(0, |acc, flag| acc | flag),
            can_boost: self.can_boost,
            counterable: self.counterable,
            time_freeze: self.time_freeze,
            skip_time_freeze_intro: self.skip_time_freeze_intro,
            meta_classes: self.meta_classes.clone(),
        }
    }
}

impl<'lua> rollback_mlua::FromLua<'lua> for CardProperties {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        _lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        let table = match lua_value {
            rollback_mlua::Value::Table(table) => table,
            _ => {
                return Err(rollback_mlua::Error::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "CardProperties",
                    message: None,
                })
            }
        };

        Ok(CardProperties {
            package_id: table.get("package_id").unwrap_or_default(),
            code: table.get("code").unwrap_or_default(),
            short_name: table
                .get("short_name")
                .unwrap_or_else(|_| String::from("?????")),
            damage: table.get("damage").unwrap_or_default(),
            boosted_damage: table.get("boosted_damage").unwrap_or_default(),
            element: table.get("element").unwrap_or_default(),
            secondary_element: table.get("secondary_element").unwrap_or_default(),
            card_class: table.get("card_class").unwrap_or_default(),
            hit_flags: table.get("hit_flags").unwrap_or_default(),
            can_boost: table.get("can_boost").unwrap_or(true),
            counterable: table.get("counterable").unwrap_or(true),
            time_freeze: table.get("time_freeze").unwrap_or_default(),
            skip_time_freeze_intro: table.get("skip_time_freeze_intro").unwrap_or_default(),
            meta_classes: table.get("meta_classes").unwrap_or_default(),
        })
    }
}

impl<'lua> rollback_mlua::ToLua<'lua> for CardProperties {
    fn to_lua(
        self,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<rollback_mlua::Value<'lua>> {
        <&CardProperties>::to_lua(&self, lua)
    }
}

impl<'lua> rollback_mlua::ToLua<'lua> for &CardProperties {
    fn to_lua(
        self,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<rollback_mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("package_id", self.package_id.as_str())?;
        table.set("code", self.code.as_str())?;
        table.set("short_name", self.short_name.as_str())?;
        table.set("damage", self.damage)?;
        table.set("boosted_damage", self.boosted_damage)?;
        table.set("element", self.element)?;
        table.set("secondary_element", self.secondary_element)?;
        table.set("card_class", self.card_class)?;
        table.set("hit_flags", self.hit_flags)?;
        table.set("can_boost", self.can_boost)?;
        table.set("counterable", self.counterable)?;
        table.set("time_freeze", self.time_freeze)?;
        table.set("skip_time_freeze_intro", self.skip_time_freeze_intro)?;
        table.set("meta_classes", self.meta_classes.clone())?;

        Ok(rollback_mlua::Value::Table(table))
    }
}
