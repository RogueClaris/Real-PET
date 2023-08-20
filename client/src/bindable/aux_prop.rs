use super::{Comparison, Element, HitFlags, HitProperties, MathExpr};
use crate::battle::{BattleCallback, Entity};
use crate::render::FrameTime;
use packets::structures::Emotion;

#[derive(Clone)]
pub enum AuxVariable {
    MaxHealth,
    Health,
    Damage,
}

impl AuxVariable {
    pub fn create_resolver(
        health: i32,
        max_health: i32,
        damage: i32,
    ) -> impl Fn(&AuxVariable) -> f32 + Copy {
        move |var| match var {
            AuxVariable::MaxHealth => max_health as f32,
            AuxVariable::Health => health as f32,
            AuxVariable::Damage => damage as f32,
        }
    }
}

impl<'lua> rollback_mlua::FromLua<'lua> for AuxVariable {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        let name: rollback_mlua::String = lua.unpack(lua_value.clone())?;
        let name_str = name.to_str()?;

        match name_str {
            "MAX_HEALTH" => Ok(AuxVariable::MaxHealth),
            "HEALTH" => Ok(AuxVariable::Health),
            "DAMAGE" => Ok(AuxVariable::Damage),
            _ => Err(rollback_mlua::Error::RuntimeError(format!(
                "Invalid AuxProp variable: {name_str:?}"
            ))),
        }
    }
}

#[derive(Clone)]
pub enum AuxRequirement {
    Interval(FrameTime),
    HitElement(Element),
    HitElementIsWeakness,
    HitFlag(HitFlags),
    HitDamage(Comparison, i32),
    ProjectedHitDamage(MathExpr<f32, AuxVariable>, Comparison, i32),
    TotalDamage(Comparison, i32),
    Element(Element),
    Emotion(Emotion),
    ProjectedHPThreshold(MathExpr<f32, AuxVariable>, Comparison, f32),
    ProjectedHP(MathExpr<f32, AuxVariable>, Comparison, i32),
    HPThreshold(Comparison, f32),
    HP(Comparison, i32),
}

impl AuxRequirement {
    // always activates when applicable, granted that no additional conditions are required or provided.
    const UNCONDITIONAL_PRIORITY: usize = 0;
    // apply every X real-time frames, otherwise treated as unconditional.
    const TIMER_PRIORITY: usize = 1;
    // activates when matching specific properties from incoming hitboxes.
    const HIT_PRIORITY: usize = 2;
    // properties of self
    const BODY_PRIORITY: usize = 3;
    // activates when current HP matches a specified formula that is allowed to be more open ended. Could allow for cases such as "HP is above 50% of max", "HP has a 4 in it", "HP is under 20% of max", "HP is less than 10". Cannot check the amount of incoming damage.
    const HP_EXPR_PRIOIRTY: usize = 4;
    // just like HP EXPR but do not activate if the specified threshold is reached
    const HP_GE_PRIORITY: usize = 5;
    // activates if HP would dip below a specified percentage threshold if the currently evaluated damage were to be applied. 0% HP is a valid threshold here, meaning HP would reach zero this turn. These checks will still be run even if no damage is being applied this frame. Can also compare with arbitrarily provided numbers, ie "would my HP reach 0% if I reduced it by 1"
    const HP_LE_PRIORITY: usize = 6;

    fn priority(&self) -> usize {
        match self {
            AuxRequirement::Interval(_) => Self::TIMER_PRIORITY, // TIMER
            AuxRequirement::HitElement(_)
            | AuxRequirement::HitElementIsWeakness
            | AuxRequirement::HitFlag(_)
            | AuxRequirement::HitDamage(_, _)
            | AuxRequirement::ProjectedHitDamage(_, _, _)
            | AuxRequirement::TotalDamage(_, _) => Self::HIT_PRIORITY, // HIT
            AuxRequirement::Element(_) | AuxRequirement::Emotion(_) => Self::BODY_PRIORITY, // BODY
            AuxRequirement::ProjectedHPThreshold(_, _, _)
            | AuxRequirement::ProjectedHP(_, _, _) => Self::HP_EXPR_PRIOIRTY, // HP EXPR
            AuxRequirement::HPThreshold(cmp, _) | AuxRequirement::HP(cmp, _) => match cmp {
                Comparison::GT | Comparison::GE => Self::HP_GE_PRIORITY, // HP GE
                _ => Self::HP_LE_PRIORITY,                               // HP LE
            },
        }
    }

    fn is_hit_requirement(&self) -> bool {
        self.priority() == Self::HIT_PRIORITY
    }
}

impl<'lua> rollback_mlua::FromLua<'lua> for AuxRequirement {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        let table: rollback_mlua::Table = lua.unpack(lua_value.clone())?;

        let name: rollback_mlua::String = table.get(1)?;
        let name_str = name.to_str()?;

        let requirement = match name_str {
            "require_interval" => AuxRequirement::Interval(table.get(2)?),
            "require_hit_element" => AuxRequirement::HitElement(table.get(2)?),
            "require_hit_element_is_weakness" => AuxRequirement::HitElementIsWeakness,
            "require_hit_flag" => AuxRequirement::HitFlag(table.get(2)?),
            "require_hit_damage" => AuxRequirement::HitDamage(table.get(2)?, table.get(3)?),
            "require_projected_hit_damage" => {
                AuxRequirement::ProjectedHitDamage(table.get(2)?, table.get(3)?, table.get(4)?)
            }
            "require_total_damage" => AuxRequirement::TotalDamage(table.get(2)?, table.get(3)?),
            "require_element" => AuxRequirement::Element(table.get(2)?),
            "require_emotion" => AuxRequirement::Emotion(table.get(2)?),
            "require_projected_health_threshold" => {
                AuxRequirement::ProjectedHPThreshold(table.get(2)?, table.get(3)?, table.get(4)?)
            }
            "require_projected_health" => {
                AuxRequirement::ProjectedHP(table.get(2)?, table.get(3)?, table.get(4)?)
            }
            "require_health_threshold" => AuxRequirement::HPThreshold(table.get(2)?, table.get(3)?),
            "require_health" => AuxRequirement::HP(table.get(2)?, table.get(3)?),
            _ => {
                return Err(rollback_mlua::Error::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "AuxRequirement",
                    message: None,
                });
            }
        };

        Ok(requirement)
    }
}

#[derive(Default, Clone)]
pub enum AuxEffect {
    StatusImmunity(HitFlags),
    ApplyStatus(HitFlags, FrameTime),
    RemoveStatus(HitFlags),
    IncreaseHitDamage(MathExpr<f32, AuxVariable>),
    DecreaseHitDamage(MathExpr<f32, AuxVariable>),
    DecreaseDamageSum(MathExpr<f32, AuxVariable>),
    DrainHP(i32),
    RecoverHP(i32),
    #[default]
    None,
}

impl AuxEffect {
    fn priority(&self) -> usize {
        match self {
            AuxEffect::StatusImmunity(_) => 0,
            AuxEffect::ApplyStatus(_, _) => 1,
            AuxEffect::RemoveStatus(_) => 2,
            AuxEffect::IncreaseHitDamage(_) => 3,
            AuxEffect::DecreaseHitDamage(_) => 4,
            AuxEffect::DecreaseDamageSum(_) => 5,
            AuxEffect::DrainHP(_) => 6,
            AuxEffect::RecoverHP(_) => 7,
            AuxEffect::None => 8,
        }
    }

    pub fn execute_before_hit(&self) -> bool {
        self.priority() < 3
    }

    pub fn execute_on_hit(&self) -> bool {
        (3..=4).contains(&self.priority())
    }

    pub fn execute_after_hit(&self) -> bool {
        self.priority() > 4
    }
}

impl<'lua> rollback_mlua::FromLua<'lua> for AuxEffect {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        if matches!(lua_value, rollback_mlua::Nil) {
            return Ok(AuxEffect::None);
        }

        let table: rollback_mlua::Table = lua.unpack(lua_value.clone())?;

        let name: rollback_mlua::String = table.get(1)?;
        let name_str = name.to_str()?;

        let effect = match name_str {
            "declare_immunity" => AuxEffect::StatusImmunity(table.get(2)?),
            "apply_status" => AuxEffect::ApplyStatus(table.get(2)?, table.get(3)?),
            "remove_status" => AuxEffect::RemoveStatus(table.get(2)?),
            "increase_hit_damage" => AuxEffect::IncreaseHitDamage(table.get(2)?),
            "decrease_hit_damage" => AuxEffect::DecreaseHitDamage(table.get(2)?),
            "decrease_total_damage" => AuxEffect::DecreaseDamageSum(table.get(2)?),
            "drain_hp" => AuxEffect::DrainHP(table.get(2)?),
            "recover_hp" => AuxEffect::RecoverHP(table.get(2)?),
            _ => {
                return Err(rollback_mlua::Error::FromLuaConversionError {
                    from: lua_value.type_name(),
                    to: "AuxEffect",
                    message: None,
                });
            }
        };

        Ok(effect)
    }
}

#[derive(Default, Clone)]
struct RequirementState {
    passed: bool,
    tested: bool,
}

#[derive(Clone)]
pub struct AuxProp {
    requirements: Vec<(AuxRequirement, RequirementState)>,
    effect: AuxEffect,
    callbacks: Vec<BattleCallback>,
    deletes_on_activation: bool,
    deletes_next_run: bool,
    tested: bool,
}

impl AuxProp {
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
            effect: AuxEffect::None,
            callbacks: Vec::new(),
            deletes_on_activation: false,
            deletes_next_run: false,
            tested: false,
        }
    }

    pub fn new_element_bonus(element: Element) -> Self {
        Self::new()
            .with_requirement(AuxRequirement::HitElement(element))
            .with_effect(AuxEffect::IncreaseHitDamage(
                MathExpr::from_variable(AuxVariable::Damage).mul(2.0),
            ))
    }

    pub fn with_effect(mut self, effect: AuxEffect) -> Self {
        assert!(matches!(self.effect, AuxEffect::None));
        self.effect = effect;
        self
    }

    pub fn with_callback(mut self, callback: BattleCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    pub fn with_requirement(mut self, requirement: AuxRequirement) -> Self {
        self.requirements
            .push((requirement, RequirementState::default()));
        self
    }

    pub fn delete_next_run(mut self) -> Self {
        self.deletes_next_run = true;
        self
    }

    pub fn completed(&self) -> bool {
        (self.deletes_next_run && self.tested)
            || (self.deletes_on_activation && self.passed_all_tests())
    }

    pub fn effect(&self) -> &AuxEffect {
        &self.effect
    }

    pub fn callbacks(&self) -> &[BattleCallback] {
        &self.callbacks
    }

    pub fn priority(&self) -> (usize, usize) {
        let effect_priority = self.effect.priority();
        let requirements_priority = self
            .requirements
            .iter()
            .map(|(requirement, _)| requirement.priority())
            .max()
            .unwrap_or(AuxRequirement::UNCONDITIONAL_PRIORITY);

        (effect_priority, requirements_priority)
    }

    pub fn mark_tested(&mut self) {
        self.tested = self.requirements.iter().all(|(_, state)| state.tested);
    }

    pub fn reset_tests(&mut self) {
        for (_, state) in &mut self.requirements {
            *state = RequirementState::default();
        }
    }

    pub fn passed_all_tests(&self) -> bool {
        self.requirements.iter().all(|(_, state)| state.passed)
    }

    pub fn hit_passes_tests(
        &mut self,
        entity: &Entity,
        health: i32,
        max_health: i32,
        hit_props: &HitProperties,
    ) -> bool {
        let previously_passing = self.passed_all_tests();
        self.process_hit(entity, health, max_health, hit_props);
        let now_passing = self.passed_all_tests();

        // reset
        if previously_passing {
            for (_, state) in &mut self.requirements {
                state.passed = true;
            }
        }

        now_passing
    }

    pub fn process_body(&mut self, emotion: &Emotion, element: Element) {
        for (requirement, state) in &mut self.requirements {
            let result = match requirement {
                AuxRequirement::Element(elem) => *elem == element,
                AuxRequirement::Emotion(emot) => emot == emotion,
                _ => continue,
            };

            state.tested = true;
            state.passed = result;
        }
    }

    pub fn process_time(&mut self, time: FrameTime) {
        for (requirement, state) in &mut self.requirements {
            let AuxRequirement::Interval(n) = requirement else {
                continue;
            };

            state.tested = true;
            state.passed = time % *n == 0;
        }
    }

    pub fn process_hit(
        &mut self,
        entity: &Entity,
        health: i32,
        max_health: i32,
        hit_props: &HitProperties,
    ) {
        for (requirement, state) in &mut self.requirements {
            let result = match requirement {
                AuxRequirement::HitElement(element) => {
                    hit_props.element == *element || hit_props.secondary_element == *element
                }
                AuxRequirement::HitElementIsWeakness => {
                    entity.element.is_weak_to(hit_props.element)
                        || entity.element.is_weak_to(hit_props.secondary_element)
                }
                AuxRequirement::HitFlag(hit_flag) => hit_props.flags & *hit_flag == *hit_flag,
                AuxRequirement::HitDamage(cmp, damage) => cmp.compare(hit_props.damage, *damage),
                AuxRequirement::ProjectedHitDamage(expr, cmp, damage) => {
                    let value = expr.eval(AuxVariable::create_resolver(
                        health,
                        max_health,
                        hit_props.damage,
                    ));

                    cmp.compare(value, *damage as f32)
                }
                _ => continue,
            };

            state.tested = true;
            state.passed = result;
        }
    }

    pub fn process_health_calculations(&mut self, health: i32, max_health: i32, total_damage: i32) {
        let get_var = AuxVariable::create_resolver(health, max_health, total_damage);

        for (requirement, state) in &mut self.requirements {
            if state.passed {
                // skip if we've already processed, occurs for hit and prehit aux props
                continue;
            }

            let result = match requirement {
                AuxRequirement::TotalDamage(cmp, reference) => {
                    cmp.compare(total_damage, *reference)
                }
                AuxRequirement::ProjectedHPThreshold(expr, cmp, reference) => {
                    let value = expr.eval(get_var);

                    cmp.compare(value / max_health as f32, *reference)
                }
                AuxRequirement::ProjectedHP(expr, cmp, reference) => {
                    let value = expr.eval(get_var);

                    cmp.compare(value, *reference as f32)
                }
                AuxRequirement::HPThreshold(cmp, reference) => {
                    cmp.compare(health as f32 / max_health as f32, *reference)
                }
                AuxRequirement::HP(cmp, reference) => cmp.compare(health, *reference),
                _ => continue,
            };

            state.tested = true;
            state.passed = result;
        }
    }
}

impl<'lua> rollback_mlua::FromLua<'lua> for AuxProp {
    fn from_lua(
        lua_value: rollback_mlua::Value<'lua>,
        lua: &'lua rollback_mlua::Lua,
    ) -> rollback_mlua::Result<Self> {
        let table: rollback_mlua::Table = lua.unpack(lua_value)?;

        let effect = table.raw_get("effect")?;
        let callbacks = table.raw_get("callbacks").unwrap_or_default();
        let deletes_on_activation = table.raw_get("delete_on_activate").unwrap_or_default();
        let deletes_next_run = table.raw_get("delete_next_run").unwrap_or_default();

        let requirements_table: rollback_mlua::Table = table.raw_get("requirements")?;
        let requirements_len = requirements_table.raw_len() as usize;
        let mut requirements = Vec::with_capacity(requirements_len);

        for requirement in requirements_table.raw_sequence_values::<AuxRequirement>() {
            requirements.push((requirement?, RequirementState::default()));
        }

        Ok(AuxProp {
            requirements,
            effect,
            callbacks,
            deletes_on_activation,
            deletes_next_run,
            tested: false,
        })
    }
}
