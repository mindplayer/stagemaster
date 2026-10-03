#include "PreviewSceneActor.h"
#include "Components/StaticMeshComponent.h"
#include "Components/SpotLightComponent.h"

void APreviewSceneActor::RotateVisual(FPreviewFixtureVisual& Visual, double Yaw)
{
    if (Yaw == 0) return;
    const FVector Origin = Visual.Light->GetComponentLocation();
    const FQuat Rotation(FVector::UpVector, FMath::DegreesToRadians(-Yaw));
    for (auto Part : {Visual.Body, Visual.Lens, Visual.Base, Visual.ArmLeft, Visual.ArmRight}) if (Part)
        Part->SetWorldLocationAndRotation(Origin + Rotation.RotateVector(Part->GetComponentLocation() - Origin), Rotation * Part->GetComponentQuat());
    Visual.Light->SetWorldRotation(Rotation * Visual.Light->GetComponentQuat());
}
bool APreviewSceneActor::PreviewPosition(const FString& Id, const FVector& Location)
{
    return PreviewTransform(Id, Location, 0);
}
bool APreviewSceneActor::PreviewTransform(const FString& Id, const FVector& Location, double Yaw)
{
    auto Visual = Fixtures.Find(Id);
    if (!FrameValid || !Visual || !FindFixture(Id) || Location.ContainsNaN() || Location.GetAbsMax() > 10000000 || !FMath::IsFinite(Yaw)) return false;
    RotateVisual(*Visual, -Visual->DraftYaw);
    const FVector Delta = Location - Visual->Light->GetComponentLocation();
    for (auto Part : {Visual->Body, Visual->Lens, Visual->Base, Visual->ArmLeft, Visual->ArmRight}) if (Part) Part->AddWorldOffset(Delta);
    Visual->Light->SetWorldLocation(Location);
    RotateVisual(*Visual, Yaw);
    Visual->DraftYaw = Yaw;
    return true;
}
void APreviewSceneActor::RestorePosition(const FString& Id)
{
    const auto Fixture = FindFixture(Id);
    const auto Visual = Fixtures.Find(Id);
    if (!Fixture || !Visual) return;
    RotateVisual(*Visual, -Visual->DraftYaw);
    Visual->DraftYaw = 0;
    const FVector Delta = Fixture->Origin - Visual->Light->GetComponentLocation();
    for (auto Part : {Visual->Body, Visual->Lens, Visual->Base, Visual->ArmLeft, Visual->ArmRight}) if (Part) Part->AddWorldOffset(Delta);
    Visual->Light->SetWorldLocation(Fixture->Origin);
}
FVector APreviewSceneActor::PreviewLocation(const FString& Id) const
{
    const auto Visual = Fixtures.Find(Id);
    return Visual ? Visual->Light->GetComponentLocation() : FVector::ZeroVector;
}
FVector APreviewSceneActor::PreviewDirection(const FString& Id) const
{
    const auto Fixture = FindFixture(Id);
    const auto Visual = Fixtures.Find(Id);
    return Fixture ? Fixture->Direction.RotateAngleAxis(Visual ? -Visual->DraftYaw : 0, FVector::UpVector) : FVector::ZeroVector;
}
