#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewSelection.h"
#include "PreviewTransform.h"
#include "GameFramework/PlayerController.h"

bool APreviewCameraPawn::PointOnDragPlane(const FVector2D& Screen, FVector& Point) const
{
    auto Player = Cast<APlayerController>(GetController());
    FVector Origin, Direction;
    return Player && Player->DeprojectScreenPositionToWorld(static_cast<float>(Screen.X), static_cast<float>(Screen.Y), Origin, Direction)
        && StageMaster::DragPlanePoint(Origin, Direction, DragOrigin, DragPlaneNormal, Point);
}
void APreviewCameraPawn::RestoreFixtures(const TArray<FString>& Ids)
{
    if (Scene) for (const auto& Id : Ids) Scene->RestorePosition(Id);
}
void APreviewCameraPawn::ClearPendingPlacement()
{
    InteractionMessage.Empty();
    RestoreFixtures(PendingFixtures);
    PendingPlacement.Empty();
    PendingFixtures.Empty();
    PendingDelta = FVector::ZeroVector;
}
void APreviewCameraPawn::PlacementResult(const FString& Id, bool Accepted)
{
    if (Id != PendingPlacement) return;
    if (!Accepted)
    {
        ClearPendingPlacement();
        InteractionMessage = TEXT("整组灯位未应用，请检查桌面提示");
    }
}
void APreviewCameraPawn::DragTo(const FVector2D& Screen)
{
    if (!Dragging || !Scene) return;
    if (!Scene->CanMoveFixtures() || DragSerial != Scene->GetSceneSerial()) { CancelDrag(); return; }
    if (!DragMoved && (Screen - DragScreenOrigin).SizeSquared() < 9) return;
    if (Tool == TEXT("rotate") || Tool == TEXT("scale"))
    {
        const double DragDistance = Screen.X - DragScreenOrigin.X;
        PreviewTransform(Tool == TEXT("rotate") ? FMath::Clamp(DragDistance * 0.5, -360.0, 360.0) : 0,
            Tool == TEXT("scale") ? FMath::Clamp(FMath::Exp(DragDistance / 200.0), 0.01, 100.0) : 1);
        return;
    }
    FVector Point;
    if (!PointOnDragPlane(Screen, Point)) return;
    FVector Delta = Point + DragOffset - DragOrigin;
    if (IsVerticalMove()) Delta.X = Delta.Y = 0;
    else Delta.Z = 0;
    for (const auto& Id : DragIds)
    {
        const auto Fixture = Scene->FindFixture(Id);
        if (!Fixture || (Fixture->Origin + Delta).ContainsNaN() || (Fixture->Origin + Delta).GetAbsMax() > 10000000)
        {
            CancelDrag();
            InteractionMessage = TEXT("整组移动超出允许范围，已恢复原位");
            return;
        }
    }
    for (const auto& Id : DragIds)
    {
        if (!Scene->PreviewPosition(Id, Scene->FindFixture(Id)->Origin + Delta)) { CancelDrag(); return; }
    }
    DragPosition = DragOrigin + Delta;
    DragMoved = true;
}
void APreviewCameraPawn::FinishDrag()
{
    if (!Dragging || !Scene) return;
    const bool Transforming = Tool == TEXT("rotate") || Tool == TEXT("scale");
    const bool Unchanged = Transforming ? (FMath::Fmod(DragYaw, 360.0) == 0 && DragScale == 1) : DragPosition.Equals(DragOrigin, 0.00001);
    if (!DragMoved || Unchanged || DragSerial != Scene->GetSceneSerial() || !Scene->CanMoveFixtures())
    {
        CancelDrag();
        return;
    }
    auto Proposal = Transforming ? StageMaster::TransformRequest(Scene->GetScene().Stamp, DragIds, DragYaw, DragScale)
        : StageMaster::TranslationRequest(Scene->GetScene().Stamp, DragIds, DragPosition - DragOrigin);
    if (!Proposal.IsValid())
    {
        CancelDrag();
        InteractionMessage = TEXT("整组灯位超出允许范围，已恢复原位");
        return;
    }
    Proposal->SetStringField(TEXT("kind"), Transforming ? TEXT("transform") : TEXT("translation"));
    PendingPlacement = FGuid::NewGuid().ToString(EGuidFormats::Digits).ToLower();
    PendingFixtures = DragIds;
    PendingDelta = DragPosition - DragOrigin;
    PendingSerial = Scene->GetSceneSerial();
    PendingUntil = FPlatformTime::Seconds() + 3;
    Proposal->SetStringField(TEXT("requestId"), PendingPlacement);
    Dragging = DragMoved = false;
    DragIds.Empty();
    if (!Streaming.Send(Proposal)) ClearPendingPlacement();
}
void APreviewCameraPawn::CancelDrag()
{
    InteractionMessage.Empty();
    if (Dragging) RestoreFixtures(DragIds);
    Dragging = DragMoved = false;
    DragIds.Empty();
}
