#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewSelection.h"
#include "DrawDebugHelpers.h"
#include "GameFramework/PlayerController.h"

void APreviewCameraPawn::TickSelection()
{
    if (!PendingPlacement.IsEmpty() && (PendingSerial != Scene->GetSceneSerial() ||
        !Scene->CanMoveFixtures() || FPlatformTime::Seconds() > PendingUntil)) ClearPendingPlacement();
    if (ProjectId != Scene->GetScene().ProjectId)
    {
        CancelDrag();
        ClearPendingPlacement();
        ProjectId = Scene->GetScene().ProjectId;
        // Host selection can arrive before the new scene. Missing identities never select another fixture.
        FocusAll();
    }
    if (Dragging && (!Scene->CanMoveFixtures() || DragSerial != Scene->GetSceneSerial()))
    {
        CancelDrag();
        InteractionMessage = TEXT("灯位拖动已取消：连接或工程状态变化");
    }
    if (Marquee.Active && (!Scene->CanMoveFixtures() || Marquee.Serial != Scene->GetSceneSerial())) CancelDrag();
    for (const auto& Id : SelectedIds) if (const auto Fixture = Scene->FindFixture(Id))
    {
        const FVector Position = Scene->PreviewLocation(Id);
        const FColor Color = Id == SelectedIds.Last() ? FColor(255, 211, 124) : FColor(124, 168, 255);
        DrawDebugSphere(GetWorld(), Position, 17, 16, Color, false, 0, 1, 1.5f);
        DrawDebugDirectionalArrow(GetWorld(), Position, Position + Scene->PreviewDirection(Id) * 80, 10, Color, false, 0, 1, 1.5f);
    }
}
void APreviewCameraPawn::SelectAt(const FVector2D& Screen, bool Additive, bool Remove)
{
    CancelDrag();
    InteractionMessage.Empty();
    auto Player = Cast<APlayerController>(GetController());
    if (!Player || !Scene || !PendingPlacement.IsEmpty()) return;
    if (!MoveMode)
    {
        if (Scene->CanMoveFixtures()) Marquee.Begin(Screen, Scene->GetSceneSerial(), Additive, Remove, MarqueeMode);
        return;
    }
    Additive |= Remove;
    FHitResult Hit;
    Player->GetHitResultAtScreenPosition(Screen, ECC_Visibility, true, Hit);
    FString Id = Scene->FixtureAt(Hit);
    // Selected handles stay usable among overlapping bodies or behind scenery.
    // Only movement mode gives them priority; ordinary picking keeps scene depth.
    if (MoveMode && !Additive)
    {
        double Nearest = 14 * 14;
        for (const auto& Selected : SelectedIds) if (const auto Candidate = Scene->FindFixture(Selected))
        {
            FVector2D Projected;
            if (Player->ProjectWorldLocationToScreen(Candidate->Origin, Projected))
            {
                const double Squared = (Projected - Screen).SizeSquared();
                if (Squared <= Nearest) { Id = Selected; Nearest = Squared; }
            }
        }
    }
    const auto After = StageMaster::SelectFixture(SelectedIds, Id, Additive, MoveMode);
    PublishSelection(After);
    const auto Fixture = Scene->FindFixture(Id);
    if (!MoveMode || Additive || !Fixture) return;
    if (SelectedIds.Num() > 256)
    {
        InteractionMessage = TEXT("一次最多移动 256 台灯具，请缩小选择范围");
        return;
    }
    if (!Scene->CanMoveFixtures())
    {
        InteractionMessage = TEXT("场地尚未同步，请稍后再移动灯位");
        return;
    }
    for (const auto& Selected : SelectedIds) if (!Scene->FindFixture(Selected)) return;
    DragOrigin = Fixture->Origin;
    DragIds = SelectedIds;
    DragSerial = Scene->GetSceneSerial();
    DragScreenOrigin = Screen;
    if (Tool == TEXT("rotate") || Tool == TEXT("scale"))
    {
        if (PrepareTransform()) { Dragging = true; DragMoved = false; }
        return;
    }
    DragPlaneNormal = IsVerticalMove() ? GetActorForwardVector().GetSafeNormal2D() : FVector::UpVector;
    FVector Point;
    if (!PointOnDragPlane(Screen, Point) || (IsVerticalMove() && FMath::Abs(GetActorForwardVector().Z) > 0.999))
    {
        InteractionMessage = IsVerticalMove() ? TEXT("升降请切换到透视视图") : TEXT("当前镜头过于平直，请俯视后拖动");
        return;
    }
    DragIds = SelectedIds;
    DragOffset = DragOrigin - Point;
    DragPosition = DragOrigin;
    DragScreenOrigin = Screen;
    DragSerial = Scene->GetSceneSerial();
    Dragging = true;
    DragMoved = false;
}
void APreviewCameraPawn::SelectFromHost(const TArray<FString>& Ids)
{
    if (Ids == SelectedIds) return;
    CancelDrag();
    ClearPendingPlacement();
    SelectedIds = Ids;
}
FText APreviewCameraPawn::SelectionText() const
{
    if (Scene && !SelectedIds.IsEmpty()) if (const auto Fixture = Scene->FindFixture(SelectedIds.Last()))
    {
        const FVector Location = Scene->PreviewLocation(Fixture->Id);
        const auto Position = StageMaster::ToMeters(Location);
        return FText::FromString(FString::Printf(TEXT("已选 %d 台 · %s   位置 %.3f / %.3f / %.3f 米"),
            SelectedIds.Num(), *Fixture->Name, Position.X, Position.Y, Position.Z));
    }
    return FText::FromString(TEXT("未选择灯具"));
}
