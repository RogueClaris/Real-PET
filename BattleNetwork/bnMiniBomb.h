#pragma once
#include "bnSpell.h"

class MiniBomb : public Spell {
private:
  int random;
  sf::Vector2f start;
  float arcDuration;
  float arcProgress;
  float cooldown;
  float damageCooldown;

public:
  MiniBomb(Field* _field, Team _team, sf::Vector2f startPos, float _duration, int damage);
  ~MiniBomb();

  void OnUpdate(float _elapsed) override;
  bool Move(Direction _direction) override;
  void Attack(Character* _entity) override;
};