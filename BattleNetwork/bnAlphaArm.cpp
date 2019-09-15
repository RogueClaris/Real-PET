#include "bnAlphaArm.h"
#include "bnMobMoveEffect.h"
#include "bnDefenseIndestructable.h"
#include "bnTile.h"
#include "bnTextureResourceManager.h"
#include "bnShaderResourceManager.h"
#include "bnAudioResourceManager.h"
#include <Swoosh\Ease.h>

#define RESOURCE_PATH "resources/mobs/alpha/alpha.animation"

AlphaArm::AlphaArm(Field* _field, Team _team, AlphaArm::Type type)
  : Obstacle(_field, _team), isMoving(false), type(type) {
  this->setScale(2.f, 2.f);
  this->SetFloatShoe(true);
  this->SetTeam(_team);
  this->SetDirection(Direction::LEFT);
  this->EnableTileHighlight(false);

  this->SetHealth(999);

  this->SetSlideTime(sf::seconds(0.1333f)); // 8 frames

  Hit::Properties props = Hit::DefaultProperties;
  props.flags |= Hit::recoil | Hit::breaking;
  props.damage = 120;
  this->SetHitboxProperties(props);

  AddDefenseRule(new DefenseIndestructable());

  shadow = new SpriteSceneNode();
  shadow->setTexture(LOAD_TEXTURE(MISC_SHADOW));
  shadow->SetLayer(1);

  totalElapsed = 0;

  this->setTexture(LOAD_TEXTURE(MOB_ALPHA_ATLAS));
  auto animComponent = (AnimationComponent*)RegisterComponent(new AnimationComponent(this));
  animComponent->Setup(RESOURCE_PATH);
  animComponent->Load();

  blueShadow = new SpriteSceneNode();
  blueShadow->setTexture(LOAD_TEXTURE(MOB_ALPHA_ATLAS));
  blueShadow->SetLayer(1);

  Animation blueShadowAnim(animComponent->GetFilePath());
  blueShadowAnim.Load();
  blueShadowAnim.SetAnimation("LEFT_CLAW_SWIPE_SHADOW");
  blueShadowAnim.Update(0, *blueShadow);

  switch (type) {
  case Type::LEFT_IDLE:
    animComponent->SetAnimation("LEFT_CLAW_DEFAULT");
    this->AddNode(shadow);
    break;
  case Type::RIGHT_IDLE:
    animComponent->SetAnimation("RIGHT_CLAW_DEFAULT");
    this->AddNode(shadow);
    break;
  case Type::LEFT_SWIPE:
    animComponent->SetAnimation("LEFT_CLAW_SWIPE");
    SetSlideTime(sf::seconds(0.13f)); // 8 frames in 60 seconds
    SetDirection(Direction::DOWN);
    this->AddNode(shadow);

    blueArmShadowPos = {
      sf::Vector2f(0, -80.0f), sf::Vector2f(0, -10.0f),
      sf::Vector2f(0, -80.0f), sf::Vector2f(0, -10.0f),
     sf::Vector2f(0, -80.0f), sf::Vector2f(0, -10.0f),  sf::Vector2f(0, -40.0f)
    };

    blueArmShadowPosIdx = 0;
    blueShadowTimer = 0.0f;

    blueShadow->setPosition(0, -10.0f);
    blueShadow->Hide();
    this->AddNode(blueShadow);
    break;
  case Type::RIGHT_SWIPE:
    animComponent->SetAnimation("RIGHT_CLAW_SWIPE");
    SetSlideTime(sf::seconds(0.13f)); // 8 frames in 60 seconds
    SetDirection(Direction::LEFT);
    changeState = (rand() % 10 < 5) ? TileState::POISON : TileState::ICE;
    break;
  }

  isSwiping = false;

  animComponent->OnUpdate(0);
}

AlphaArm::~AlphaArm() {
  // this->RemoveNode(shadow);
  delete shadow;
  delete blueShadow;
}

bool AlphaArm::CanMoveTo(Battle::Tile * next)
{
  return true;
}

void AlphaArm::OnUpdate(float _elapsed) {
  totalElapsed += _elapsed;
  float delta = std::sinf(10*totalElapsed+1);


  if (type == Type::LEFT_SWIPE) {
    blueShadowTimer += _elapsed;
    
    if (blueShadowTimer > 1.0f / 60.0f) {
      blueShadowTimer = 0;
      blueArmShadowPosIdx++;
    }

    blueArmShadowPosIdx = blueArmShadowPosIdx % blueArmShadowPos.size();

    blueShadow->setPosition(blueArmShadowPos[blueArmShadowPosIdx]);

    delta = GetTile()->GetHeight();

    if (totalElapsed > 1.0f) {
      if (!isSwiping) {
        isSwiping = true;
        AUDIO.Play(AudioType::SWORD_SWING);
      }

      blueShadow->Reveal();

      delta = (1.0f - swoosh::ease::linear(totalElapsed - 1.0f, 0.12f, 1.0f)) * GetTile()->GetHeight();

      if (totalElapsed - 1.0f > 0.12f) {
        // May have just finished moving
        this->GetTile()->AffectEntities(this);

        // Keep moving
        if (!this->IsSliding()) {
          this->SlideToTile(true);
          this->Move(this->GetDirection());

          if (!GetNextTile()) {
            this->Delete();
          }
        }
      }
    }
  }
  else if (type == Type::RIGHT_SWIPE) {
    delta = 0; // do not bob

      // May have just finished moving
    this->GetTile()->AffectEntities(this);

    if (totalElapsed > 1.2f) {
      if (!isSwiping) {
        isSwiping = true;
        AUDIO.Play(AudioType::TOSS_ITEM_LITE);
      }

      if (!Teammate(GetTile()->GetTeam()) && GetTile()->IsWalkable()) {
        GetTile()->SetState(changeState);
      }

      // Keep moving
      if (!this->IsSliding()) {
        this->SlideToTile(true);
        this->Move(this->GetDirection());

        if (!GetNextTile()) {
          this->Delete();
        }
      }
    }
  }

  setPosition(tile->getPosition().x + tileOffset.x, tile->getPosition().y + tileOffset.y - GetHitHeight() - delta);

  shadow->setPosition(-13, -4 + delta / 2.0f); // counter offset the shadow node

}

void AlphaArm::OnDelete() {
  auto fx = new MobMoveEffect(GetField());
  GetField()->AddEntity(*fx, GetTile()->GetX(), GetTile()->GetY());
}

const bool AlphaArm::OnHit(const Hit::Properties props) {
  return false;
}

void AlphaArm::Attack(Character* other) {
  Obstacle* isObstacle = dynamic_cast<Obstacle*>(other);

  if (isObstacle) {
    auto props = Hit::DefaultProperties;
    props.damage = 9999;
    isObstacle->Hit(props);
    this->hit = true;
    return;
  }

  Character* isCharacter = dynamic_cast<Character*>(other);

  if (isCharacter && isCharacter != this) {
    isCharacter->Hit(GetHitboxProperties());
  }
}

const float AlphaArm::GetHitHeight() const
{
  switch (type) {
  case Type::LEFT_IDLE:
    return 10; break;
  case Type::RIGHT_IDLE:
    return 10; break;
  case Type::LEFT_SWIPE:
    return 30; break;
  case Type::RIGHT_SWIPE:
    return 10; break;
  }

  return 0;
}

const bool AlphaArm::IsSwiping() const
{
  return isSwiping;
}

void AlphaArm::SyncElapsedTime(const float elapsedTime)
{
  // the claws get out of sync, we must sync them up
  this->totalElapsed = elapsedTime;
}
