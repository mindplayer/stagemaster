#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewTransform.h"

bool APreviewCameraPawn::PrepareTransform()
{
    if (!Scene || !Scene->CanMoveFixtures() || DragIds.IsEmpty() || DragIds.Num() > 256) return false;
    FBox Bounds(ForceInit);
    for (const auto& Id : DragIds)
    {
        const auto Fixture = Scene->FindFixture(Id);
        if (!Fixture) return false;
        Bounds += Fixture->Origin;
    }
    DragCenter = Bounds.GetCenter();
    DragOrigin = DragPosition = DragCenter;
    DragYaw = 0;
    DragScale = 1;
    return true;
}
bool APreviewCameraPawn::PreviewTransform(double Angle, double Scale)
{
    if (!Scene || DragSerial != Scene->GetSceneSerial() || !Scene->CanMoveFixtures()) { CancelDrag(); return false; }
    // Match the six-decimal proposal, so the visual never previews a different transform.
    Angle = FMath::RoundToDouble(Angle * 1000000) / 1000000;
    Scale = FMath::RoundToDouble(Scale * 1000000) / 1000000;
    TArray<FVector> Positions;
    for (const auto& Id : DragIds)
    {
        const auto Fixture = Scene->FindFixture(Id);
        if (!Fixture) { CancelDrag(); return false; }
        const auto Next = StageMaster::TransformPoint(Fixture->Origin, DragCenter, Angle, Scale);
        if (Next.ContainsNaN() || Next.GetAbsMax() > 10000000)
        {
            CancelDrag(); InteractionMessage = TEXT("整组变换超出场地范围，已恢复原位"); return false;
        }
        Positions.Add(Next);
    }
    for (int32 Index = 0; Index < DragIds.Num(); ++Index)
        if (!Scene->PreviewTransform(DragIds[Index], Positions[Index], Angle)) { CancelDrag(); return false; }
    DragYaw = Angle;
    DragScale = Scale;
    DragMoved = true;
    InteractionMessage = Tool == TEXT("rotate") ? FString::Printf(TEXT("整组旋转 %.2f°"), Angle)
        : FString::Printf(TEXT("间距比例 %.3f"), Scale);
    return true;
}
void APreviewCameraPawn::TransformExact(const TArray<FString>& Ids, double Angle, double Scale)
{
    if (!MoveMode || !Scene || Dragging || !PendingPlacement.IsEmpty() || Ids != SelectedIds ||
        (Tool != TEXT("rotate") && Tool != TEXT("scale"))) return;
    DragIds = Ids;
    DragSerial = Scene->GetSceneSerial();
    if (!PrepareTransform()) { DragIds.Empty(); return; }
    Dragging = true;
    DragMoved = false;
    if (PreviewTransform(Angle, Scale)) FinishDrag();
}
