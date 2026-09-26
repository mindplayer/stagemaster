#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Pawn.h"
#include "PreviewStreaming.h"
#include "PreviewCameraPawn.generated.h"

class UCameraComponent;
class APreviewSceneActor;
class SWidget;

UCLASS()
class APreviewCameraPawn : public APawn
{
    GENERATED_BODY()
public:
    APreviewCameraPawn();
    virtual void Tick(float DeltaSeconds) override;
    virtual void EndPlay(const EEndPlayReason::Type Reason) override;
    void ViewAction(const FString& Action);
    void Navigate(const FVector2D& Delta, bool Pan);
    void Zoom(float Steps);
    void FocusAll();
    void FocusSelected();
    void TopView();
    void SelectAt(const FVector2D& Screen);
    void SelectFromHost(const FString& Id);
    void PlacementResult(const FString& Id, bool Accepted);
    bool IsMoveMode() const { return MoveMode; }
    void DragTo(const FVector2D& Screen);
    void FinishDrag();
    void CancelDrag();
    void ToggleMove();
    void ToggleWorkLight();
    FText StatusText() const;
    FText SelectionText() const;
    FText MoveText() const;
    FText WorkLightText() const;
protected:
    virtual void BeginPlay() override;
private:
    void UpdateCamera();
    bool PointOnDragPlane(const FVector2D& Screen, FVector& Point) const;
    void ClearPendingPlacement();
    UPROPERTY() TObjectPtr<UCameraComponent> Camera;
    UPROPERTY() TObjectPtr<APreviewSceneActor> Scene;
    TSharedPtr<SWidget> Overlay;
    FPreviewStreaming Streaming;
    FString SelectedId;
    FString ProjectId;
    FString InteractionMessage;
    FVector Pivot = FVector::ZeroVector;
    FVector DragOrigin;
    FVector DragOffset;
    FVector DragPosition;
    FVector2D DragScreenOrigin;
    double Distance = 2000;
    float Yaw = -45;
    float Pitch = -35;
    uint64 DragSerial = 0;
    bool MoveMode = false;
    bool Dragging = false;
    bool DragMoved = false;
    FString PendingPlacement;
    FString PendingFixture;
    double PendingUntil = 0;
    uint64 PendingSerial = 0;
};
