#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "PreviewProtocol.h"
#include "PreviewSceneActor.generated.h"

class UProceduralMeshComponent;
class UStaticMeshComponent;
class USpotLightComponent;
class UDirectionalLightComponent;
class UMaterialInterface;
class UMaterialInstanceDynamic;
class UStaticMesh;
namespace StageMaster { class FPreviewBridge; }

USTRUCT()
struct FPreviewFixtureVisual
{
    GENERATED_BODY()
    UPROPERTY() TObjectPtr<UStaticMeshComponent> Body;
    UPROPERTY() TObjectPtr<USpotLightComponent> Light;
    UPROPERTY() TObjectPtr<UStaticMeshComponent> Lens;
    UPROPERTY() TObjectPtr<UMaterialInstanceDynamic> LensMaterial;
};

// UE owns only disposable visual components. All edits return through the bridge.
UCLASS()
class APreviewSceneActor : public AActor
{
    GENERATED_BODY()
public:
    APreviewSceneActor();
    virtual void Tick(float DeltaSeconds) override;
    virtual void EndPlay(const EEndPlayReason::Type Reason) override;
    const StageMaster::FScene& GetScene() const { return Current; }
    const FString& GetStatus() const { return Status; }
    bool CanMoveFixtures() const { return CanEdit; }
    uint64 GetSceneSerial() const { return SceneSerial; }
    FBox GetBounds() const;
    FString FixtureAt(const FHitResult& Hit) const;
    const StageMaster::FFixture* FindFixture(const FString& Id) const;
    bool PreviewPosition(const FString& Id, const FVector& Location);
    void RestorePosition(const FString& Id);
    bool CommitPosition(const FString& Id, const FVector& Location);
    void ToggleWorkLight();
    bool HasWorkLight() const;
protected:
    virtual void BeginPlay() override;
private:
    void ApplyScene(StageMaster::FScene&& Scene);
    void ApplyFrame(StageMaster::FFrame&& Frame);
    void Invalidate(const FString& Reason);
    void ClearVisuals();
    UPROPERTY() TObjectPtr<UMaterialInterface> SurfaceMaterial;
    UPROPERTY() TObjectPtr<UStaticMesh> FixtureMesh;
    UPROPERTY() TObjectPtr<UStaticMesh> LensMesh;
    UPROPERTY() TObjectPtr<UMaterialInterface> LensMaterial;
    UPROPERTY() TObjectPtr<UDirectionalLightComponent> WorkLight;
    UPROPERTY() TMap<FString, TObjectPtr<UProceduralMeshComponent>> Meshes;
    UPROPERTY() TMap<FString, FPreviewFixtureVisual> Fixtures;
    TSharedPtr<StageMaster::FPreviewBridge> Bridge;
    StageMaster::FScene Current;
    FString Status = TEXT("正在连接舞台大师");
    uint64 SceneSerial = 0;
    bool CanEdit = false;
};
