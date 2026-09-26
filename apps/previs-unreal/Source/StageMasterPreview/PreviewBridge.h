#pragma once

#include "CoreMinimal.h"
#include "Interfaces/IHttpRequest.h"
#include "PreviewProtocol.h"

namespace StageMaster
{
class FPreviewBridge : public TSharedFromThis<FPreviewBridge>
{
public:
    ~FPreviewBridge() { Stop(); }
    TFunction<void(FScene&&)> OnScene;
    TFunction<void(FFrame&&)> OnFrame;
    TFunction<void(const FString&)> OnInvalidated;

    bool Start();
    void Stop();
    void Tick(double Now);
    bool SubmitPlacement(const TSharedPtr<FJsonObject>& Request);
    bool IsBusy() const { return Pending.IsValid(); }
    const FString& GetMessage() const { return Message; }

private:
    enum class ERequest { Scene, Frame, Placement };
    void Request(ERequest Kind, const TSharedPtr<FJsonObject>& Body = nullptr);
    void Complete(ERequest Kind, FHttpRequestPtr Request, FHttpResponsePtr Response, bool Success);
    void Fail(const FString& Reason, double RetryAfter = 0.5);
    bool AcceptScene(const TSharedPtr<FJsonObject>& Object);
    bool AcceptFrame(const TSharedPtr<FJsonObject>& Object);

    FString BaseUrl;
    FString Authorization;
    FString ExpectedBridgeId;
    FString Message;
    FString Version;
    uint32 Generation = 0;
    TSet<FString> FixtureIds;
    FHttpRequestPtr Pending;
    ERequest PendingKind = ERequest::Scene;
    double NextRequestAt = 0;
    double LastFrameAt = 0;
    bool Running = false;
    bool NeedScene = true;
    bool WasValid = false;
};
}
