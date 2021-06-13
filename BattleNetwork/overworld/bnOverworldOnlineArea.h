#pragma once

#include <Poco/Net/DatagramSocket.h>
#include <Poco/Buffer.h>
#include <map>
#include <unordered_map>
#include <functional>

#include "../netplay/bnBufferReader.h"
#include "bnOverworldSceneBase.h"
#include "bnOverworldPacketProcessor.h"
#include "bnOverworldActorPropertyAnimator.h"
#include "bnPacketHeaders.h"
#include "bnServerAssetManager.h"

namespace Overworld {
  struct OnlinePlayer {
    OnlinePlayer(std::string name) : actor(std::make_shared<Overworld::Actor>(name)) {}

    std::shared_ptr<Overworld::Actor> actor;
    Overworld::EmoteNode emoteNode;
    Overworld::TeleportController teleportController{};
    bool disconnecting{ false };
    Direction idleDirection;
    sf::Vector3f startBroadcastPos{};
    sf::Vector3f endBroadcastPos{};
    long long timestamp{};
    std::array<double, NetManager::LAG_WINDOW_LEN> lagWindow{ 0 };
    ActorPropertyAnimator propertyAnimator;
    size_t packets{};
  };

  class OnlineArea final : public SceneBase {
  private:
    struct ExcludedObjectData {
      bool visible;
      bool solid;
    };

    struct AssetMeta {
      std::string name;
      std::string shortName;
      uint64_t lastModified{};
      bool cachable{};
      AssetType type{};
      size_t size{};
      Poco::Buffer<char> buffer{ 0 };
    };

    std::string ticket; //!< How we are represented on the server
    Poco::Net::SocketAddress remoteAddress; //!< server
    std::shared_ptr<PacketProcessor> packetProcessor;
    std::string connectData;
    uint16_t maxPayloadSize;
    bool isConnected{ false };
    bool transferringServers{ false };
    bool kicked{ false };
    bool isEnteringBattle{ false };
    bool tryPopScene{ false };
    ActorPropertyAnimator propertyAnimator;
    SelectedNavi lastFrameNavi{};
    ServerAssetManager serverAssetManager;
    AssetMeta incomingAsset;
    std::map<std::string, OnlinePlayer> onlinePlayers;
    std::map<unsigned, ExcludedObjectData> excludedObjects;
    std::vector<std::vector<TileObject*>> warps;
    std::list<std::string> removePlayers;
    sf::Vector3f lastPosition;
    Timer movementTimer;
    Text transitionText;
    Text nameText;
    std::optional<std::string> trackedPlayer;
    bool wasReadingTextBox{ false };
    CameraController serverCameraController;
    CameraController warpCameraController;

    void detectWarp();
    bool positionIsInWarp(sf::Vector3f position);
    Overworld::TeleportController::Command& teleportIn(sf::Vector3f position, Direction direction);
    void transferServer(const std::string& address, uint16_t port, const std::string& data, bool warpOut);
    void processPacketBody(const Poco::Buffer<char>& data);

    void sendAssetFoundSignal(const std::string& path, uint64_t lastModified);
    void sendAssetsFound();
    void sendAssetStreamSignal(ClientAssetType assetType, uint16_t headerSize, const char* data, size_t size);
    void sendLoginSignal();
    void sendLogoutSignal();
    void sendRequestJoinSignal();
    void sendReadySignal();
    void sendTransferredOutSignal();
    void sendCustomWarpSignal(unsigned int tileObjectId);
    void sendPositionSignal();
    void sendAvatarChangeSignal();
    void sendAvatarAssetStream();
    void sendEmoteSignal(const Overworld::Emotes emote);
    void sendObjectInteractionSignal(unsigned int tileObjectId);
    void sendNaviInteractionSignal(const std::string& ticket);
    void sendTileInteractionSignal(float x, float y, float z);
    void sendTextBoxResponseSignal(char response);
    void sendPromptResponseSignal(const std::string& response);
    void sendBoardOpenSignal();
    void sendBoardCloseSignal();
    void sendPostRequestSignal();
    void sendPostSelectSignal(const std::string& postId);

    void receiveLoginSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveTransferWarpSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveTransferStartSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveTransferCompleteSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveTransferServerSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveKickSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveAssetRemoveSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveAssetStreamStartSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveAssetStreamSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receivePreloadSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveCustomEmotesPathSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveMapSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receivePlaySoundSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveExcludeObjectSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveIncludeObjectSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveMoveCameraSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveSlideCameraSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveShakeCameraSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveTrackWithCameraSignal(BufferReader& reader, const Poco::Buffer<char>& buffer);
    void receiveTeleportSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveMessageSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveQuestionSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveQuizSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receivePromptSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveOpenBoardSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receivePrependPostsSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveAppendPostsSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveRemovePostSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveCloseBBSSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receivePVPSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorConnectedSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorDisconnectedSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorSetNameSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorMoveSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorSetAvatarSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorEmoteSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorAnimateSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void receiveActorKeyFramesSignal(BufferReader& reader, const Poco::Buffer<char>&);
    void leave();
  protected:
    virtual std::string GetPath(const std::string& path);
    virtual std::string GetText(const std::string& path);
    virtual std::shared_ptr<sf::Texture> GetTexture(const std::string& path);
    virtual std::shared_ptr<sf::SoundBuffer> GetAudio(const std::string& path);


  public:
    /**
     * @brief Loads the player's library data and loads graphics
     */
    OnlineArea(
      swoosh::ActivityController&,
      const std::string& address,
      uint16_t port,
      const std::string& connectData,
      uint16_t maxPayloadSize,
      bool guestAccount
    );

    /**
    * @brief deconstructor
    */
    ~OnlineArea();

    void onUpdate(double elapsed) override;
    void onDraw(sf::RenderTexture& surface) override;
    void onStart() override;
    void onEnd() override;
    void onLeave() override;
    void onResume() override;

    void OnTileCollision() override;
    void OnInteract() override;
    void OnEmoteSelected(Overworld::Emotes emote) override;
  };
}