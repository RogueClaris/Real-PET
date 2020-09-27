#pragma once

#include <memory>
#include <type_traits>
#include <vector>
#include <map>
#include <SFML/Graphics/RenderTexture.hpp>
#include <Swoosh/ActivityController.h>
#include <Swoosh/Activity.h>
#include <Swoosh/Timer.h>
#include <Segues/WhiteWashFade.h>
#include <Segues/BlackWashFade.h>

#include "../bnCounterHitListener.h"
#include "../bnCharacterDeleteListener.h"
#include "../bnCardUseListener.h"
#include "../bnComponent.h"
#include "../bnMobHealthUI.h"
#include "../bnAnimation.h"
#include "../bnCamera.h"
#include "../bnCounterCombatRule.h"
#include "../bnPlayerCardUseListener.h"
#include "../bnEnemyCardUseListener.h"
#include "../bnSelectedCardsUI.h"

// Battle scene specific classes
#include "bnBattleSceneState.h"
#include "States/bnFadeOutBattleState.h"

// forward declare statements
class Field; 
class Player;
class Mob;
class Player;
class PlayerHealthUI;
class CounterCombatRule;
class Background;

// using namespaces
using sf::RenderWindow;
using sf::VideoMode;
using sf::Clock;
using sf::Event;
using sf::Font;

// Combos are counted if more than one enemy is hit within x frames
// The game is clocked to display 60 frames per second
// If x = 20 frames, then we want a combo hit threshold of 20/60 = 0.3 seconds
#define COMBO_HIT_THRESHOLD_SECONDS 20.0f/60.0f

/**
  @brief BattleSceneBase class provides an API for creating complex states
*/
class BattleSceneBase : public swoosh::Activity, public CounterHitListener, public CharacterDeleteListener, public CardUseListener {
private:
    bool quitting{false}; //!< Determine if we are leaving the battle scene

    // general stuff
    double elapsed{ 0 }; /*!< total time elapsed in battle */
    int round{ 0 }; //!< Some scene types repeat battles and need to track rounds
    Field* field{ nullptr }; /*!< Supplied by mob info: the grid to battle on */
    Player* player{ nullptr }; /*!< Pointer to player's selected character */
    SelectedCardsUI cardUI; /*!< Player's Card UI implementation */
    PlayerCardUseListener cardListener; /*!< Card use listener handles one card at a time */
    EnemyCardUseListener enemyCardListener; /*!< Enemies can use cards now */
    std::vector<std::string> mobNames; /*!< List of every non-deleted mob spawned */
    Camera camera; /*!< Camera object - will shake screen */
    Background* background; /*!< Custom backgrounds provided by Mob data */
    int randBG; /*!< If background provided by Mob data is nullptr, randomly select one */
    std::shared_ptr<sf::Font> font; /*!< PAUSE font */
    sf::Text* pauseLabel; /*!< "PAUSE" text */
    std::shared_ptr<sf::Texture> customBarTexture; /*!< Cust gauge image */
    SpriteProxyNode customBarSprite; /*!< Cust gauge sprite */
    sf::Vector2f customBarPos; /*!< Cust gauge position */
    std::shared_ptr<sf::Font> mobFont; /*!< Name of mob font */
    sf::Sprite mobEdgeSprite, mobBackdropSprite; /*!< name backdrop images*/
    Mob* mob; /*!< Mob and mob data player are fighting against */
    double customProgress{ 0 }; /*!< Cust bar progress */
    double customDuration; /*!< Cust bar max time */
    bool didDoubleDelete{ false }; /*!< Flag if player double deleted this frame */
    bool didTripleDelete{ false }; /*!< Flag if player tripled deleted this frame */
    double backdropOpacity{ 1.0 };
    std::vector<SceneNode*> scenenodes; /*!< Scene node system */

    // counter stuff
    SpriteProxyNode counterReveal;
    Animation counterRevealAnim;
    CounterCombatRule* counterCombatRule{ nullptr };

    // card stuff
    Battle::Card** cards; /*!< List of Card* the user selects from the card cust */
    int cardCount; /*!< Length of card list */

    // sprites
    sf::Sprite doubleDelete; /*!< "Double Delete" graphic */
    sf::Sprite tripleDelete; /*!< "Triple Delete" graphic */
    sf::Sprite counterHit; /*!< "Counter Hit" graphic */
    sf::Sprite comboInfo;  /*!< double delete and triple delete placeholder. Only one appears at a time */
    sf::Vector2f comboInfoPos; /*!< Position of comboInfo on screen */
    swoosh::Timer comboInfoTimer; /*!< How long the info should stay on screen */
    swoosh::Timer multiDeleteTimer; /*!< Deletions start a 12 frame timer to count towards combos */
    swoosh::Timer battleTimer; /*!< Total duration of active battle time */

    // shader fx
    double shaderCooldown;
    sf::Shader& pauseShader; /*!< Dim screen */
    sf::Shader& whiteShader; /*!< Fade out white */
    sf::Shader& yellowShader; /*!< Turn tiles yellow */
    sf::Shader& customBarShader; /*!< Cust gauge shaders */
    sf::Shader& heatShader; /*!< Heat waves and red hue */
    sf::Shader& iceShader; /*!< Reflection in the ice */
    sf::Texture& distortionMap; /*!< Distortion effect pixel sample source */
    sf::Vector2u textureSize; /*!< Size of distorton effect */

protected:
    using ChangeCondition = BattleSceneState::ChangeCondition;

    /*
      \brief StateNode represents a node in a graph of conditional states

      States can flow from one to another that it is linked to.
      We call two linked nodes an Edge.
      To transition from one node to the other, the linked condition must be met (true).
      We can link battle scene states together in the inherited BattleSceneBase class.
    */
    class StateNode {
      friend class BattleSceneBase;

      BattleSceneState& state; //!< The battle scene state this node represents
      BattleSceneBase& owner; //!< The scene this state refers to
    public:
      StateNode(BattleSceneBase& owner, BattleSceneState& state) 
      : state(state), owner(owner)
      {}
    };

    /*
      \brief This wrapper is just a StateNode with a few baked-in API functions to create easy conditional state transitions 
    */
    template<typename T>
    class StateNodeWrapper : public StateNode {
      T& state;

      public:
      using Class = T;

      StateNodeWrapper(BattleSceneBase& owner, T& state) 
      : state(state), StateNode(owner, state)
      {}

      /* 
          \brief Return the underlining state object as a pointer
      */
      T* operator->() {
        return &state;
      }

      /*
          \brief Return the underlining state object pointer as a reference
      */
      T& operator*() {
        return state;
      }

      /*
          \brief if input functor is a member function, then create a closure to call the function on a class object 
      */
      template<
        typename MemberFunc,
        typename = typename std::enable_if<std::is_member_function_pointer<MemberFunc>::value>::type
      >
      StateNodeWrapper& ChangeOnEvent(StateNode& next, MemberFunc when) {
        T* statePtr = &state;
        owner.Link(*this, next, 
          [statePtr, when]{
              return (statePtr->*when)();
          });
        return *this;
      }

      /* 
          \brief if input is a lambda, use the ::Link() API function already provided
      */
      template<
        typename Lambda,
        typename = typename std::enable_if<!std::is_member_function_pointer<Lambda>::value>::type
      >
      StateNodeWrapper& ChangeOnEvent(StateNode& next, const Lambda& when) {
        owner.Link(*this, next, when);
        return *this;
      }
    };


    /**
     * @brief Get the total number of counter moves
     * @return const int
     */
    const int GetCounterCount() const;

    void HandleCounterLoss(Character& subject);

    /**
   * @brief Scans the entity list for updated components and tries to Inject them if the components require.
   */
    void ProcessNewestComponents();

    /**
     * @brief State boolean for BattleScene. Query if the battle is over.
     * @return true if isPostBattle is true, otherwise false
     */
    const bool IsCleared();

    /**
     * @brief Query if the battle update loop is ticking.
     * @return true if the field is not paused
    */
    const bool IsBattleActive();

    /**
      @brief Crude support card filter step
    */
    void FilterSupportCards(Battle::Card** cards, int cardCount);

#ifdef __ANDROID__
    void SetupTouchControls();
    void ShutdownTouchControls();

    bool releasedB;
#endif

public:

    BattleSceneBase(swoosh::ActivityController* controller, Player* localPlayer);
    virtual ~BattleSceneBase();

    /*
        \brief Use class type T as the state and perfect-forward arguments to the class 
        \return StateNodeWrapper<T> structure for easy programming
    */
    template<typename T, typename... Args>
    StateNodeWrapper<T> AddState(Args&&... args) {
      T* ptr = new T(std::forward<decltype(args)>(args)...);
      states.insert(states.begin(), ptr);
      return StateNodeWrapper<T>(*this, *ptr);
      using Class = T;
    }

    /*  
        \brief Set the current state pointer to this state node reference and begin the scene
    */
    void StartStateGraph(StateNode& start);

    /*
        \brief Update the scene and current state. If any conditions are satisfied, transition to the linked state
    */
    virtual void onUpdate(double elapsed) override;

    virtual void onDraw(sf::RenderTexture& surface) override;

    /*
        \brief Forces the creation a fadeout state onto the state pointer and goes back to the last scene
    */
    void Quit(const FadeOut& mode);

    /**
   * @brief Inject uses double-visitor design pattern. Battle Scene subscribes to card pub components.
   * @param pub CardUsePublisher component to subscribe to
   */
    void Inject(CardUsePublisher& pub);

    /**
     * @brief Inject uses double-visitor design pattern. BattleScene adds component to draw and update list.
     * @param other
     */
    void Inject(MobHealthUI& other);

    /**
     * @brief Inject uses double-visitor design pattern. This is default case.
     * @param other Adds component "other" to component update list.
     */
    void Inject(Component* other);

    /**
     * @brief When ejecting component from scene, simply removes it from update list
     * @param other
     */
    void Eject(Component::ID_t ID);

private:
    BattleSceneState* current{nullptr}; //!< Pointer to the current battle scene state
    std::vector<BattleSceneState*> states; //!< List of all battle scene states

    /*
        \brief Edge represents a link from from state A to state B when a condition is met
    */
    struct Edge {
      std::reference_wrapper<StateNode> a; 
      std::reference_wrapper<StateNode> b; 
      ChangeCondition when; //!< functor that returns boolean
    };

    std::multimap<BattleSceneState*, Edge> nodeToEdges; //!< All edges. Together, they form a graph

    /*
        \brief Creates an edge with a condition and packs it away into the scene graph
    */
    void Link(StateNode& a, StateNode& b, ChangeCondition&& when);
};
