#include "PreviewSceneActor.h"
#include "PreviewObjects.h"
#include "ProceduralMeshComponent.h"

const StageMaster::FFixture* APreviewSceneActor::SelectionFixture(const FString& Key) const
{
    return Key.StartsWith(TEXT("p:")) ? FindFixture(Key.Mid(2)) : nullptr;
}
TArray<FString> APreviewSceneActor::SelectionKeys(bool AllObjects) const
{
    TArray<FString> Keys;
    for (const auto& Fixture : Current.Fixtures) Keys.Add(StageMaster::FixtureKey(Fixture.Id));
    if (AllObjects) for (const auto& Mesh : Current.Meshes)
        if (!Mesh.ConstructionId.IsEmpty() && !(Cutaway && Mesh.EnclosureShell)) Keys.AddUnique(StageMaster::ConstructionKey(Mesh.ConstructionId));
    return Keys;
}
FString APreviewSceneActor::ObjectAt(const FHitResult& Hit, bool AllObjects) const
{
    const auto Fixture = FixtureAt(Hit);
    if (!Fixture.IsEmpty()) return StageMaster::FixtureKey(Fixture);
    if (AllObjects && Hit.GetComponent()) for (const auto& Mesh : Current.Meshes)
        if (!Mesh.ConstructionId.IsEmpty() && Meshes.FindRef(Mesh.Id) == Hit.GetComponent()) return StageMaster::ConstructionKey(Mesh.ConstructionId);
    return {};
}
FBox APreviewSceneActor::ObjectBounds(const FString& Key, bool Draft) const
{
    FBox Bounds(ForceInit);
    if (const auto Fixture = SelectionFixture(Key)) { Bounds += Draft ? PreviewLocation(Fixture->Id) : Fixture->Origin; return Bounds; }
    if (!Key.StartsWith(TEXT("c:"))) return Bounds;
    for (const auto& Mesh : Current.Meshes) if (Mesh.ConstructionId == Key.Mid(2))
    {
        const auto Component = Meshes.FindRef(Mesh.Id);
        const FVector Delta = Draft && Component ? Component->GetRelativeLocation() : FVector::ZeroVector;
        for (const auto& Vertex : Mesh.Vertices) Bounds += Vertex + Delta;
    }
    return Bounds;
}
FString APreviewSceneActor::ObjectName(const FString& Key) const
{
    if (const auto Fixture = SelectionFixture(Key)) return Fixture->Name;
    for (const auto& Mesh : Current.Meshes) if (Key == StageMaster::ConstructionKey(Mesh.ConstructionId)) return Mesh.Name;
    return {};
}
bool APreviewSceneActor::CanTranslateObjects(const TArray<FString>& Keys) const
{
    if (Keys.IsEmpty() || Keys.Num() > 256) return false;
    for (const auto& Key : Keys)
    {
        if (SelectionFixture(Key)) continue;
        const auto Mesh = Current.Meshes.FindByPredicate([&](const auto& M) { return Key == StageMaster::ConstructionKey(M.ConstructionId); });
        if (!Mesh || !Mesh->Movable || !ObjectBounds(Key, false).IsValid) return false;
    }
    return true;
}
TArray<FString> APreviewSceneActor::AffectedFixtures(const TArray<FString>& Keys) const
{
    auto Ids = StageMaster::FixtureIds(Keys);
    for (const auto& Mesh : Current.Meshes) if (Keys.Contains(StageMaster::ConstructionKey(Mesh.ConstructionId)))
        for (const auto& Id : Mesh.AttachedFixtureIds) Ids.AddUnique(Id);
    return Ids;
}
bool APreviewSceneActor::PreviewObjects(const TArray<FString>& Keys, const FVector& Delta)
{
    if (!FrameValid || !CanTranslateObjects(Keys) || Delta.ContainsNaN()) return false;
    const auto Ids = AffectedFixtures(Keys);
    for (const auto& Id : Ids)
    {
        const auto Fixture = FindFixture(Id);
        if (!Fixture || (Fixture->Origin + Delta).GetAbsMax() > 10000000) return false;
    }
    for (const auto& Key : Keys)
    {
        const auto Box = ObjectBounds(Key, false);
        if (!Box.IsValid || (Box.Min + Delta).GetAbsMax() > 11000000 || (Box.Max + Delta).GetAbsMax() > 11000000) return false;
    }
    for (const auto& Mesh : Current.Meshes) if (Keys.Contains(StageMaster::ConstructionKey(Mesh.ConstructionId)))
        if (auto Component = Meshes.FindRef(Mesh.Id)) Component->SetRelativeLocation(Delta);
    for (const auto& Id : Ids) if (!PreviewPosition(Id, FindFixture(Id)->Origin + Delta)) return false;
    return true;
}
void APreviewSceneActor::RestoreObjects(const TArray<FString>& Keys)
{
    for (const auto& Mesh : Current.Meshes) if (Keys.Contains(StageMaster::ConstructionKey(Mesh.ConstructionId)))
        if (auto Component = Meshes.FindRef(Mesh.Id)) Component->SetRelativeLocation(FVector::ZeroVector);
    for (const auto& Id : AffectedFixtures(Keys)) RestorePosition(Id);
}
