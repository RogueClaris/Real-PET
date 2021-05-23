#include <Swoosh/ActivityController.h>
#include <Segues/WhiteWashFade.h>
#include <Segues/PixelateBlackWashFade.h>
#include <chrono>

#include "bnNetworkBattleScene.h"
#include "../bnPlayerInputReplicator.h"
#include "../bnPlayerNetworkState.h"
#include "../bnPlayerNetworkProxy.h"
#include "../../bnFadeInState.h"

// states 
#include "states/bnNetworkSyncBattleState.h"
#include "../../battlescene/States/bnRewardBattleState.h"
#include "../../battlescene/States/bnTimeFreezeBattleState.h"
#include "../../battlescene/States/bnBattleStartBattleState.h"
#include "../../battlescene/States/bnBattleOverBattleState.h"
#include "../../battlescene/States/bnFadeOutBattleState.h"
#include "../../battlescene/States/bnCombatBattleState.h"
#include "../../battlescene/States/bnCardSelectBattleState.h"
#include "../../battlescene/States/bnCardComboBattleState.h"

// Android only headers
#include "../../Android/bnTouchArea.h"

// modals like card cust and battle reward slide in 12px per frame for 10 frames. 60 frames = 1 sec
// modal slide moves 120px in 1/6th of a second
// Per 1 second that is 6*120px in 6*1/6 of a sec = 720px in 1 sec
#define MODAL_SLIDE_PX_PER_SEC 720.0f

// Combos are counted if more than one enemy is hit within x frames
// The game is clocked to display 60 frames per second
// If x = 20 frames, then we want a combo hit threshold of 20/60 = 0.3 seconds
#define COMBO_HIT_THRESHOLD_SECONDS 20.0f/60.0f

using namespace swoosh::types;
using swoosh::ActivityController;

NetworkBattleScene::NetworkBattleScene(ActivityController& controller, const NetworkBattleSceneProps& props) : 
  BattleSceneBase(controller, props.base),
  remoteAddress(props.netconfig.remote) 
{
  auto* clientPlayer = &props.base.player;

  networkCardUseListener = new NetworkCardUseListener(*this, *clientPlayer);
  networkCardUseListener->Subscribe(this->GetSelectedCardsUI());

  selectedNavi = props.netconfig.myNavi;
  props.base.player.CreateComponent<PlayerInputReplicator>(clientPlayer);

  packetProcessor = std::make_shared<PVP::PacketProcessor>(remoteAddress, *this);
  Net().AddHandler(remoteAddress, packetProcessor);

  // If playing co-op, add more players to track here
  players = { clientPlayer };

  // ptr to player, form select index (-1 none), if should transform
  // TODO: just make this a struct to use across all states that need it...
  trackedForms = {
    std::make_shared<TrackedFormData>(clientPlayer, -1, false)
  };

  // trackedForms[0]->SetReadyState<PlayerControlledState>();

  // in seconds
  double battleDuration = 10.0;

  mob = new Mob(props.base.field);

  // First, we create all of our scene states
  auto syncState = AddState<NetworkSyncBattleState>(remotePlayer);
  auto cardSelect = AddState<CardSelectBattleState>(players, trackedForms);
  auto combat = AddState<CombatBattleState>(mob, players, battleDuration);
  auto combo = AddState<CardComboBattleState>(this->GetSelectedCardsUI(), props.base.programAdvance);
  auto forms = AddState<CharacterTransformBattleState>(trackedForms);
  auto battlestart = AddState<BattleStartBattleState>(players);
  auto battleover = AddState<BattleOverBattleState>(players);
  auto timeFreeze = AddState<TimeFreezeBattleState>();
  auto fadeout = AddState<FadeOutBattleState>(FadeOut::black, players); // this state requires arguments

  // Important! State transitions are added in order of priority!
  syncState.ChangeOnEvent(cardSelect, [this]() mutable {
    return remoteState.isRemoteConnected;
  });
  
  // Goto the combo check state if new cards are selected...
  cardSelect.ChangeOnEvent(combo, [=]() mutable {
    return cardSelect->SelectedNewChips() && packetProcessor->IsHandshakeComplete();
  });

  // ... else if forms were selected, go directly to forms ....
  cardSelect.ChangeOnEvent(forms, [=]() mutable {
    return cardSelect->HasForm() && packetProcessor->IsHandshakeComplete();
  });

  // ... Finally if none of the above, just start the battle
  cardSelect.ChangeOnEvent(battlestart, [=]() mutable {
    return cardSelect->OKIsPressed() && packetProcessor->IsHandshakeComplete();
  });

  // TODO: add startup lag for BattleStartState so opponents start at the same time
  // battlestart->SetWaitTime(roundStartDelay);

  // If we reached the combo state, we must also check if form transformation was next
  // or to just start the battle after
  combo.ChangeOnEvent(forms, [cardSelect, combo]() mutable {return combo->IsDone() && cardSelect->HasForm(); });
  combo.ChangeOnEvent(battlestart, &CardComboBattleState::IsDone);
  forms.ChangeOnEvent(battlestart, &CharacterTransformBattleState::IsFinished);
  battlestart.ChangeOnEvent(combat, &BattleStartBattleState::IsFinished);
  timeFreeze.ChangeOnEvent(combat, &TimeFreezeBattleState::IsOver);

  // special condition: if lost in combat and had a form, trigger the character transform states
  auto playerLosesInForm = [this] {
    const bool changeState = this->trackedForms[0]->player->GetHealth() == 0 && (this->trackedForms[0]->selectedForm != -1);

    if (changeState) {
      this->trackedForms[0]->selectedForm = -1;
      this->trackedForms[0]->animationComplete = false;
    }

    return changeState;
  };

  // Lambda event callback that captures and handles network card select screen opening
  auto onCardSelectEvent = [this]() mutable {
    if (combatPtr->PlayerRequestCardSelect()) {
      this->sendRequestedCardSelectSignal();
      return true;
    }
    else if (remoteState.openedCardWidget) {
      remoteState.openedCardWidget = false;
      return true;
    }

    return false;
  };

  // combat has multiple state interruptions based on events
  // so we can chain them together
  combat
    .ChangeOnEvent(battleover, &CombatBattleState::PlayerWon)
    //.ChangeOnEvent(forms, playerLosesInForm)
    .ChangeOnEvent(fadeout, &CombatBattleState::PlayerLost)
    .ChangeOnEvent(cardSelect, onCardSelectEvent)
    .ChangeOnEvent(timeFreeze, &CombatBattleState::HasTimeFreeze);

  battleover.ChangeOnEvent(fadeout, &BattleOverBattleState::IsFinished);

  // share some values between states
  combo->ShareCardList(&cardSelect->GetCardPtrList(), &cardSelect->GetCardListLengthAddr());

  // Some states need to know about card uses
  auto& ui = this->GetSelectedCardsUI();
  combat->Subscribe(ui);
  timeFreeze->Subscribe(ui);

  // pvp cannot pause
  combat->EnablePausing(false);

  // Some states are part of the combat routine and need to respect
  // the combat state's timers
  combat->subcombatStates.push_back(&timeFreeze.Unwrap());

  // We need to subscribe to new events later, so get a pointer to these states
  timeFreezePtr = &timeFreeze.Unwrap();
  combatPtr = &combat.Unwrap();
  syncStatePtr = &syncState.Unwrap();
  cardComboStatePtr = &combo.Unwrap();

  // this kicks-off the state graph beginning with the intro state
  this->StartStateGraph(syncState);

  sendConnectSignal(this->selectedNavi); // NOTE: this function only happens once at start
}

NetworkBattleScene::~NetworkBattleScene()
{ 
  delete remotePlayer;
  if (remoteHand) delete[] remoteHand;
}

void NetworkBattleScene::OnHit(Character& victim, const Hit::Properties& props)
{
}

void NetworkBattleScene::onUpdate(double elapsed) {
  
  if (!IsSceneInFocus()) return;

  BattleSceneBase::onUpdate(elapsed);

  if (!this->IsPlayerDeleted()) {
    sendHPSignal(GetPlayer()->GetHealth());
  }

  if (remotePlayer && remotePlayer->WillRemoveLater()) {
    auto iter = std::find(players.begin(), players.end(), remotePlayer);
    if (iter != players.end()) {
      players.erase(iter);
    }
    remoteState.isRemoteConnected = false;
    remotePlayer = nullptr;
  }
}

void NetworkBattleScene::onDraw(sf::RenderTexture& surface) {
  BattleSceneBase::onDraw(surface);
}

void NetworkBattleScene::onExit()
{
}

void NetworkBattleScene::onEnter()
{
}

void NetworkBattleScene::onStart()
{
  BattleSceneBase::onStart();

  // Once the transition completes, we begin handshakes
  packetProcessor->EnableKickForSilence(true);

}

void NetworkBattleScene::onResume()
{
}

void NetworkBattleScene::onEnd()
{
  Net().DropHandlers(remoteAddress);
}

void NetworkBattleScene::Inject(PlayerInputReplicator& pub)
{
  pub.Inject(*this);
}

const NetPlayFlags& NetworkBattleScene::GetRemoteStateFlags()
{
  return remoteState; 
}

void NetworkBattleScene::sendHandshakeSignal()
{
  /**
  To begin the round, we need to supply the following information to our opponent:
    1. Our selected form
    2. Our card list size
    3. Our card list items

  We need to also measure latency to synchronize client animation with
  remote animations (see: combos and forms)
  */

  int form = trackedForms[0]->selectedForm;
  auto& selectedCardsWidget = this->GetSelectedCardsUI();
  std::vector<std::string> uuids = selectedCardsWidget.GetUUIDList();
  size_t len = uuids.size();

  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals signalType{ NetPlaySignals::handshake };
  buffer.append((char*)&signalType, sizeof(NetPlaySignals));
  buffer.append((char*)&form, sizeof(int));
  buffer.append((char*)&len, sizeof(size_t));

  for (auto& id : uuids) {
    size_t len = id.size();
    buffer.append((char*)&len, sizeof(size_t));
    buffer.append(id.c_str(), len);
  }

  auto& [_, id] = packetProcessor->SendPacket(Reliability::Reliable, buffer);
  packetProcessor->UpdateHandshakeID(id);
}

void NetworkBattleScene::sendShootSignal()
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::shoot };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendUseSpecialSignal()
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::special };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendChargeSignal(const bool state)
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::charge };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&state, sizeof(bool));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendConnectSignal(const SelectedNavi navi)
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::connect };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&navi, sizeof(SelectedNavi));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendChangedFormSignal(const int form)
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::form };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&form, sizeof(int));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendHPSignal(const int hp)
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::hp };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&hp, sizeof(int));
  packetProcessor->SendPacket(Reliability::UnreliableSequenced, buffer);
}

void NetworkBattleScene::sendTileCoordSignal(const int x, const int y)
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::tile };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&x, sizeof(int));
  buffer.append((char*)&y, sizeof(int));
  packetProcessor->SendPacket(Reliability::UnreliableSequenced, buffer);
}

void NetworkBattleScene::sendChipUseSignal(const std::string& used)
{
  Logger::Logf("sending chip data over network for %s", used.data());

  uint64_t timestamp = (uint64_t)CurrentTime::AsMilli();

  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::card };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  buffer.append((char*)&timestamp, sizeof(uint64_t));
  buffer.append((char*)used.data(),used.length());
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendRequestedCardSelectSignal()
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::card_select };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::sendLoserSignal()
{
  Poco::Buffer<char> buffer{ 0 };
  NetPlaySignals type{ NetPlaySignals::loser };
  buffer.append((char*)&type, sizeof(NetPlaySignals));
  packetProcessor->SendPacket(Reliability::ReliableOrdered, buffer);
}

void NetworkBattleScene::recieveHandshakeSignal(const Poco::Buffer<char>& buffer)
{
  std::vector<std::string> remoteUUIDs;
  int remoteForm{ -1 };
  size_t cardLen{};
  size_t read{};

  std::memcpy(&remoteForm, buffer.begin(), sizeof(int));
  read += sizeof(int);

  std::memcpy(&cardLen, buffer.begin() + read, sizeof(size_t));
  read += sizeof(size_t);

  while (cardLen > 0) {
    std::string uuid;
    size_t len{};
    std::memcpy(&len, buffer.begin() + read, sizeof(size_t));
    read += sizeof(size_t);
    uuid.resize(len);
    std::memcpy(uuid.data(), buffer.begin() + read, len);
    read += len;
    remoteUUIDs.push_back(uuid);
    cardLen--;
  }

  // Now that we have the remote's form and cards,
  // populate the net play remote state with this information
  // and kick off the battle sequence

  //remoteState.remoteFormSelect = remoteForm;
  trackedForms[1]->selectedForm = remoteForm;
  trackedForms[1]->animationComplete = false; // forces animation

  if (remoteHand) delete[] remoteHand;

  remoteHand = new Battle::Card * [remoteUUIDs.size()];

  for (size_t i = 0; i < remoteUUIDs.size(); i++) {
    Battle::Card card = WEBCLIENT.MakeBattleCardFromWebCardData(WebAccounts::Card{ remoteUUIDs[i] });
    remoteHand[i] = new Battle::Card(card);
  }

  // Prepare for simulation
  cardComboStatePtr->Reset();
  double duration{};
  int remoteHandLen = int(remoteUUIDs.size());

  // simulate PA to calculate time required to animate
  while (!cardComboStatePtr->IsDone()) {
    constexpr double step = 1.0 / frame_time_t::frames_per_second;
    cardComboStatePtr->Simulate(step, &remoteHand, &remoteHandLen, false);
    duration += step;
  }

  // Prepare for next PA
  cardComboStatePtr->Reset();

  // Filter support cards
  FilterSupportCards(remoteHand, remoteHandLen);

  // Supply the final hand info
  remoteCardUsePublisher->LoadCards(remoteHand, remoteHandLen);
  
  // Convert to microseconds and use this as the round start delay
  roundStartDelay = long long(duration*1000000.0) + packetProcessor->GetAvgLatency().count();
}

void NetworkBattleScene::recieveShootSignal()
{
  if (!remoteState.isRemoteConnected) return;

  remotePlayer->Attack();
  remoteState.remoteShoot = true;
  Logger::Logf("recieved shoot signal from remote");
}

void NetworkBattleScene::recieveUseSpecialSignal()
{
  if (!remoteState.isRemoteConnected) return;

  remotePlayer->UseSpecial();
  remoteState.remoteUseSpecial = true;

  Logger::Logf("recieved use special signal from remote");

}

void NetworkBattleScene::recieveChargeSignal(const Poco::Buffer<char>& buffer)
{
  if (buffer.empty()) return;
  if (!remoteState.isRemoteConnected) return;

  bool state = remoteState.remoteCharge; 
  std::memcpy(&state, buffer.begin(), sizeof(bool));
  remoteState.remoteCharge = state;

  Logger::Logf("recieved charge signal from remote: %i", state);

}

void NetworkBattleScene::recieveConnectSignal(const Poco::Buffer<char>& buffer)
{
  if (remoteState.isRemoteConnected) return; // prevent multiple connection requests...

  remoteState.isRemoteConnected = true;

  SelectedNavi navi = SelectedNavi{ 0 }; 

  if (buffer.size() >= sizeof(SelectedNavi)) {
    std::memcpy(&navi, buffer.begin(), sizeof(SelectedNavi));
  }
  else {
    std::string str_buffer(buffer.begin(), buffer.end());
    Logger::Logf("Incoming connect signal was corrupted: %s", str_buffer.c_str());
    return;
  }

  remoteState.remoteNavi = navi;

  Logger::Logf("Recieved connect signal! Remote navi: %i", remoteState.remoteNavi);

  assert(remotePlayer == nullptr && "remote player was already set!");
  remotePlayer = NAVIS.At(navi).GetNavi();

  // TODO: manual flipping shouldn't be needed. The engine should flip based on team and direction...
  remotePlayer->SetTeam(Team::blue);
  remotePlayer->setScale(remotePlayer->getScale().x * -1.0f, remotePlayer->getScale().y);

  remotePlayer->ChangeState<FadeInState<Player>>([]{});

  GetField()->AddEntity(*remotePlayer, remoteState.remoteTileX, remoteState.remoteTileY);
  mob->Track(*remotePlayer);

  remoteCardUsePublisher = remotePlayer->CreateComponent<SelectedCardsUI>(remotePlayer);
  remoteCardUsePublisher->Hide(); // do not reveal opponent's cards
  remoteCardUseListener = new PlayerCardUseListener(*remotePlayer);
  remoteCardUseListener->Subscribe(*remoteCardUsePublisher);
  combatPtr->Subscribe(*remoteCardUsePublisher);
  timeFreezePtr->Subscribe(*remoteCardUsePublisher);

  remotePlayer->CreateComponent<MobHealthUI>(remotePlayer);
  auto netProxy = remotePlayer->CreateComponent<PlayerNetworkProxy>(remotePlayer, remoteState);

  players.push_back(remotePlayer);
  trackedForms.push_back(std::make_shared<TrackedFormData>(remotePlayer, -1, false ));
  //trackedForms.back()->SetReadyState<PlayerNetworkState>(netProxy);

  LoadMob(*mob);
}

void NetworkBattleScene::recieveChangedFormSignal(const Poco::Buffer<char>& buffer)
{
  if (buffer.empty()) return;
  int form = remoteState.remoteFormSelect;
  int prevForm = remoteState.remoteFormSelect;
  std::memcpy(&form, buffer.begin(), sizeof(int));

  if (remotePlayer && form != prevForm) {
    remoteState.remoteFormSelect = form;

    // TODO: hacky and ugly
    this->trackedForms[1]->selectedForm = remoteState.remoteFormSelect;
    this->trackedForms[1]->animationComplete = false; // kick-off animation
  }
}

void NetworkBattleScene::recieveHPSignal(const Poco::Buffer<char>& buffer)
{
  if (buffer.empty()) return;
  if (!remoteState.isRemoteConnected) return;

  int hp = remotePlayer->GetHealth();
  std::memcpy(&hp, buffer.begin(), sizeof(int));
  
  remoteState.remoteHP = hp;
  remotePlayer->SetHealth(hp);
}

void NetworkBattleScene::recieveTileCoordSignal(const Poco::Buffer<char>& buffer)
{
  if (buffer.empty()) return;
  if (!remoteState.isRemoteConnected) return;

  int x = remoteState.remoteTileX; 
  std::memcpy(&x, buffer.begin(), sizeof(int));

  int y = remoteState.remoteTileX; 
  std::memcpy(&y, (buffer.begin()+sizeof(int)), sizeof(int));

  // mirror the x value for remote
  x = (GetField()->GetWidth() - x)+1;

  remoteState.remoteTileX = x;
  remoteState.remoteTileY = y;
}

void NetworkBattleScene::recieveChipUseSignal(const Poco::Buffer<char>& buffer)
{
  if (buffer.empty()) return;
  if (!remoteState.isRemoteConnected) return;

  uint64_t timestamp = 0; std::memcpy(&timestamp, buffer.begin(), sizeof(uint64_t));
  std::string used = std::string(buffer.begin()+sizeof(uint64_t), buffer.size()-sizeof(uint64_t));
  remoteState.remoteChipUse = used;
  remoteCardUsePublisher->UseNextCard();
  Logger::Logf("remote used chip %s", used.c_str());
}

void NetworkBattleScene::recieveLoserSignal()
{
  // TODO: replace this with PVP win information
  packetProcessor->EnableKickForSilence(false);
  this->Quit(FadeOut::black);
}

void NetworkBattleScene::recieveRequestedCardSelectSignal()
{
  // also going to trigger opening the card select widget
  remoteState.openedCardWidget = true;
}

void NetworkBattleScene::processPacketBody(NetPlaySignals header, const Poco::Buffer<char>& body)
{
  try {
    switch (header) {
      case NetPlaySignals::handshake:
        recieveHandshakeSignal(body);
        break;
      case NetPlaySignals::connect:
        recieveConnectSignal(body);
        break;
      case NetPlaySignals::card:
        recieveChipUseSignal(body);
        break;
      case NetPlaySignals::form:
        recieveChangedFormSignal(body);
        break;
      case NetPlaySignals::hp:
        recieveHPSignal(body);
        break;
      case NetPlaySignals::loser:
        recieveLoserSignal();
        break;
      case NetPlaySignals::tile:
        recieveTileCoordSignal(body);
        break;
      case NetPlaySignals::shoot:
        recieveShootSignal();
        break;
      case NetPlaySignals::special:
        recieveUseSpecialSignal();
        break;
      case NetPlaySignals::charge:
        recieveChargeSignal(body);
        break;
      case NetPlaySignals::card_select:
        recieveRequestedCardSelectSignal();
        break;
    }
  }
  catch (std::exception& e) {
    Logger::Logf("PVP Network exception: %s", e.what());
    packetProcessor->HandleError();
  }
}
