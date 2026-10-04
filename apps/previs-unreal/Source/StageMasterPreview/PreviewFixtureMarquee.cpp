#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewSelection.h"
#include "PreviewObjects.h"
#include "GameFramework/PlayerController.h"
#include "Engine/World.h"

void APreviewCameraPawn::PublishSelection(const TArray<FString>& Ids)
{
    if (Ids == SelectedIds) return;
    SelectedIds = Ids;
    if (StageMaster::FixtureIds(Ids).Num() != Ids.Num() && (Tool == TEXT("rotate") || Tool == TEXT("scale"))) Tool = TEXT("horizontal");
    const auto Selection = MakeShared<FJsonObject>();
    if (ObjectsEnabled)
    {
        Selection->SetStringField(TEXT("kind"), TEXT("selectionTargets"));
        Selection->SetArrayField(TEXT("targets"), StageMaster::ObjectValues(SelectedIds));
    }
    else
    {
        Selection->SetStringField(TEXT("kind"), TEXT("selectionGroup"));
        TArray<TSharedPtr<FJsonValue>> Values;
        for (const auto& Id : StageMaster::FixtureIds(SelectedIds)) Values.Add(MakeShared<FJsonValueString>(Id));
        Selection->SetArrayField(TEXT("fixtureIds"), Values);
    }
    Streaming.Send(Selection);
}
bool APreviewCameraPawn::GetMarquee(FVector2D& Start, FVector2D& End, bool& Removing) const
{
    if (!Marquee.Active || !Marquee.Moved) return false;
    Start = Marquee.Start;
    End = Marquee.End;
    Removing = Marquee.Mode == StageMaster::EMarqueeMode::Remove;
    return true;
}
void APreviewCameraPawn::FinishMarquee()
{
    auto Player = Cast<APlayerController>(GetController());
    if (!Marquee.Active || !Scene || !Player || Marquee.Serial != Scene->GetSceneSerial() || !Scene->CanMoveFixtures())
    {
        Marquee.Cancel();
        return;
    }
    const auto Gesture = Marquee;
    Marquee.Cancel();
    if (!Gesture.Moved)
    {
        FHitResult Hit;
        Player->GetHitResultAtScreenPosition(Gesture.End, ECC_Visibility, true, Hit);
        PublishSelection(StageMaster::SelectFixture(SelectedIds, Scene->ObjectAt(Hit, ObjectsEnabled && AllObjects), Gesture.ClickAdditive, false));
        return;
    }
    int32 Width = 0, Height = 0;
    Player->GetViewportSize(Width, Height);
    TArray<FString> Hits;
    for (const auto& Key : Scene->SelectionKeys(ObjectsEnabled && AllObjects))
    {
        const auto Box = Scene->ObjectBounds(Key, false);
        if (!Box.IsValid) continue;
        const auto Origin = Box.GetCenter();
        FVector2D Projected;
        if (!Player->ProjectWorldLocationToScreen(Origin, Projected) ||
            !StageMaster::InsideMarquee(Projected, Gesture.Start, Gesture.End, FVector2D(Width, Height))) continue;
        if (!SelectionThrough)
        {
            FHitResult Hit;
            FVector RayOrigin, RayDirection;
            if (!Player->DeprojectScreenPositionToWorld(Projected.X, Projected.Y, RayOrigin, RayDirection)) continue;
            // Trace only up to the installation point. Scenery farther behind is not an occluder.
            GetWorld()->LineTraceSingleByChannel(Hit, RayOrigin, Origin, ECC_Visibility,
                FCollisionQueryParams(SCENE_QUERY_STAT(MarqueeVisibility), true));
            if (Hit.bBlockingHit && Scene->ObjectAt(Hit, true) != Key) continue;
        }
        Hits.Add(Key);
    }
    TArray<FString> Result;
    if (StageMaster::MarqueeSelection(SelectedIds, Hits, Gesture.Mode, Result)) PublishSelection(Result);
    else InteractionMessage = TEXT("框选超过对象数量限制，请缩小范围");
}
void APreviewCameraPawn::ToggleSelectionThrough()
{
    CancelDrag();
    SelectionThrough = !SelectionThrough;
}

FString APreviewCameraPawn::GetMarqueeMode() const
{
    return MarqueeMode == StageMaster::EMarqueeMode::Add ? TEXT("add") :
        MarqueeMode == StageMaster::EMarqueeMode::Remove ? TEXT("remove") : TEXT("replace");
}
