#include "bnCardComboBattleState.h"

#include "../bnBattleSceneBase.h"
#include "../../bnTextureResourceManager.h"
#include "../../bnAudioResourceManager.h"

#include <SFML/Graphics/Font.hpp>

CardComboBattleState::CardComboBattleState(SelectedCardsUI& ui, PA& programAdvance) : 
  ui(ui), 
  font(Font::Style::thick), 
  programAdvance(programAdvance) {
  /*
  Program Advance + labels
  */
  hasPA = -1;
  paStepIndex = 0;

  programAdvanceSprite = sf::Sprite(*LOAD_TEXTURE(PROGRAM_ADVANCE));
  programAdvanceSprite.setScale(2.f, 2.f);
  programAdvanceSprite.setOrigin(0, programAdvanceSprite.getLocalBounds().height / 2.0f);
  programAdvanceSprite.setPosition(40.0f, 58.f);

}

void CardComboBattleState::Simulate(double elapsed, Battle::Card*** cardsPtr, int* listLengthPtr, bool playSound)
{
  increment += 360.0 * elapsed;
  PAStartTimer.update(sf::seconds(static_cast<float>(elapsed)));

  // we're leaving a state
  // Start Program Advance checks
  if (paChecked && hasPA == -1) {
    // Filter and apply support cards
    GetScene().FilterSupportCards(*cardsPtr, *listLengthPtr);
    isPAComplete = true;
  }
  else if (!paChecked) {

    hasPA = programAdvance.FindPA(*cardsPtr, *listLengthPtr);

    if (hasPA > -1) {
      paSteps = programAdvance.GetMatchingSteps();
      PAStartTimer.reset();
      PAStartTimer.start();
    }

    paChecked = true;
  }
  else if (PAStartTimer.getElapsed().asSeconds() > PAStartLength) {

    if (listStepCounter > 0.f) {
      listStepCounter -= (float)elapsed;
    }
    else {
      // +2 = 1 step for showing PA label and 1 step for showing merged card
      // That's the cards we want to show + 1 + 1 = cardCount + 2
      if (paStepIndex == (*listLengthPtr) + 2) {

        Battle::Card* paCard = programAdvance.GetAdvanceCard();

        // Only remove the cards involved in the program advance. Replace them with the new PA card.
        // PA card is dealloc by the class that created it so it must be removed before the library tries to dealloc
        int newCardCount = (*listLengthPtr) - (int)paSteps.size() + 1; // Add the new one
        int newCardStart = hasPA;

        // Create a temp card list
        Battle::Card** newCardList = new Battle::Card * [newCardCount] {nullptr};

        int j = 0;
        for (int i = 0; i < *listLengthPtr && j < newCardCount; ) {
          if (i == hasPA) {
            newCardList[j] = paCard;
            i += (int)paSteps.size();
            j++;
            continue;
          }

          newCardList[j] = (*cardsPtr)[i];
          i++;
          j++;
        }

        // Set the new cards
        for (int i = 0; i < newCardCount; i++) {
          (*cardsPtr)[i] = newCardList[i];
        }

        // Delete the temp list space
        // We are _not_ deleting the pointers in the list, just the list itself
        delete[] newCardList;

        *listLengthPtr = newCardCount;

        hasPA = -1; // used as state reset flag
      }
      else {
        if (paStepIndex == (*listLengthPtr) + 1) {
          listStepCounter = listStepCooldown * 2.0f; // Linger on the screen when showing the final PA

          // play the sound
          if (!advanceSoundPlay && playSound) {
            Audio().Play(AudioType::PA_ADVANCE);
            advanceSoundPlay = true;
          }
        }
        else {
          listStepCounter = listStepCooldown * 0.7f; // Quicker about non-PA cards
        }

        if (paStepIndex >= hasPA && paStepIndex <= hasPA + paSteps.size() - 1) {
          listStepCounter = listStepCooldown; // Take our time with the PA cards
          
          if (playSound) {
            Audio().Play(AudioType::POINT_SFX);
          }
        }

        paStepIndex++;
      }
    }
  }
}

void CardComboBattleState::Reset()
{
  advanceSoundPlay = false;
  paChecked = false;
  isPAComplete = false;
  hasPA = -1;
  paStepIndex = 0;
  increment = 0;
  listStepCounter = listStepCooldown;
  PAStartTimer.reset();
  PAStartTimer.start();
}

void CardComboBattleState::ShareCardList(Battle::Card*** cardsPtr, int* listLengthPtr)
{
  this->cardsListPtr = cardsPtr;
  this->cardCountPtr = listLengthPtr;
}

void CardComboBattleState::onStart(const BattleSceneState*)
{
  Reset();
}

void CardComboBattleState::onEnd(const BattleSceneState*)
{
  advanceSoundPlay = false;
}

void CardComboBattleState::onUpdate(double elapsed)
{
  this->elapsed += elapsed;
  Simulate(elapsed, cardsListPtr, cardCountPtr, true);
}

void CardComboBattleState::onDraw(sf::RenderTexture& surface)
{
  if (hasPA > -1) {
    float nextLabelHeight = 0;

    double PAStartSecs = PAStartTimer.getElapsed().asSeconds();
    double scale = swoosh::ease::linear(PAStartSecs, PAStartLength, 1.0);
    programAdvanceSprite.setScale(2.f, (float)scale * 2.f);
    surface.draw(programAdvanceSprite);

    if (paStepIndex <= (*cardCountPtr) + 1) {
      for (int i = 0; i < paStepIndex && i < *cardCountPtr; i++) {
        std::string formatted = (*cardsListPtr)[i]->GetShortName();
        formatted.resize(9, ' ');
        formatted[8] = (*cardsListPtr)[i]->GetCode();

        Text stepLabel{ formatted, font };

        stepLabel.setOrigin(0, 0);
        stepLabel.setPosition(40.0f, 80.f + (nextLabelHeight * 2.f));
        stepLabel.setScale(2.0f, 2.0f);

        auto stepLabelPos = stepLabel.getPosition();
        stepLabel.setPosition(stepLabelPos + sf::Vector2f(2.f, 2.f));
        stepLabel.SetColor(sf::Color::Black);
        surface.draw(stepLabel);

        stepLabel.setPosition(stepLabelPos);

        if (i >= hasPA && i <= hasPA + paSteps.size() - 1) {
          if (i < paStepIndex - 1) {
            stepLabel.SetColor(sf::Color(128, 248, 80));
          }
          else {
            stepLabel.SetColor(sf::Color(247, 188, 27));
          }
        }
        else {
          stepLabel.SetColor(sf::Color::White);
        }

        surface.draw(stepLabel);

        // make the next label relative to this one
        nextLabelHeight += stepLabel.GetLocalBounds().height * stepLabel.getScale().y;
      }
      nextLabelHeight = 0;
    }
    else {
      for (int i = 0; i < *cardCountPtr; i++) {
        std::string formatted = (*cardsListPtr)[i]->GetShortName();
        formatted.resize(9, ' ');
        formatted[8] = (*cardsListPtr)[i]->GetCode();

        Text stepLabel{ formatted, font };

        stepLabel.setOrigin(0, 0);
        stepLabel.setPosition(40.0f, 80.f + (nextLabelHeight * 2.f));
        stepLabel.setScale(2.0f, 2.0f);
        stepLabel.SetColor(sf::Color::White);

        if (i >= hasPA && i <= hasPA + paSteps.size() - 1) {
          if (i == hasPA) {
            Battle::Card* paCard = programAdvance.GetAdvanceCard();

            Text stepLabel{ paCard->GetShortName(), font };
            stepLabel.setOrigin(0, 0);
            stepLabel.setPosition(40.0f, 80.f + (nextLabelHeight * 2.f));
            stepLabel.setScale(2.0f, 2.0f);

            auto stepLabelPos = stepLabel.getPosition();
            stepLabel.setPosition(stepLabelPos + sf::Vector2f(2.f, 2.f));
            stepLabel.SetColor(sf::Color::Black);
            surface.draw(stepLabel);

            stepLabel.setPosition(stepLabelPos);

            float rad = swoosh::ease::radians((float)increment*2.f);
            sf::Uint32 sin = static_cast<sf::Uint32>(((std::sin(rad)*0.5) + 0.5) * 150);
            sf::Uint32 sin2 = static_cast<sf::Uint32>(((std::sin(rad+swoosh::ease::pi*0.5) *0.5)+ 0.5) * 150);
            sf::Uint32 sin3 = static_cast<sf::Uint32>(((std::sin(rad+swoosh::ease::pi) * 0.5) + 0.5) * 150);

            sin += 105u;
            sin2 += 105u;
            sin3 += 105u;

            stepLabel.SetColor(sf::Color(sin, sin2, sin3));

            surface.draw(stepLabel);
          }
          else {
            // make the next label relative to the hidden one and skip drawing
            nextLabelHeight += stepLabel.GetLocalBounds().height * stepLabel.getScale().y;
            continue;
          }

        }
        else {
          auto stepLabelPos = stepLabel.getPosition();
          stepLabel.setPosition(stepLabelPos + sf::Vector2f(2.f, 2.f));
          stepLabel.SetColor(sf::Color::Black);
          surface.draw(stepLabel);

          stepLabel.setPosition(stepLabelPos);

          stepLabel.SetColor(sf::Color::White);
          surface.draw(stepLabel);
        }

        // make the next label relative to this one
        nextLabelHeight += stepLabel.GetLocalBounds().height * stepLabel.getScale().y;
      }
    }
  }
}

bool CardComboBattleState::IsDone() {
    return isPAComplete;
}