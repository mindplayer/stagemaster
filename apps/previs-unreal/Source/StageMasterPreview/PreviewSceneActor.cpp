#include "PreviewSceneActor.h"
#include "PreviewBridge.h"
#include "Components/DirectionalLightComponent.h"
#include "Components/ExponentialHeightFogComponent.h"
#include "Components/SpotLightComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "ProceduralMeshComponent.h"
#include "UObject/ConstructorHelpers.h"

namespace
{
template<typename T>
T* Attach(AActor& Owner)
{
    T* Component = NewObject<T>(&Owner);
    Owner.AddInstanceComponent(Component);
    Component->SetupAttachment(Owner.GetRootComponent());
    Component->SetMobility(EComponentMobility::Movable);
    Component->RegisterComponent();
    return Component;
}
bool SameMesh(const StageMaster::FMesh& A, const StageMaster::FMesh& B)
{
    return A.Color == B.Color && A.Vertices == B.Vertices && A.Indices == B.Indices;
}
}

APreviewSceneActor::APreviewSceneActor()
{
    PrimaryActorTick.bCanEverTick = true;
    RootComponent = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
    static ConstructorHelpers::FObjectFinder<UMaterialInterface> Material(TEXT("/Engine/BasicShapes/BasicShapeMaterial.BasicShapeMaterial"));
    static ConstructorHelpers::FObjectFinder<UStaticMesh> Body(TEXT("/DMXFixtures/LightFixtures/Meshes/SM_Static_Base.SM_Static_Base"));
    static ConstructorHelpers::FObjectFinder<UStaticMesh> Lens(TEXT("/DMXFixtures/LightFixtures/Meshes/SM_Static_Lens.SM_Static_Lens"));
    static ConstructorHelpers::FObjectFinder<UMaterialInterface> LensSurface(TEXT("/DMXFixtures/LightFixtures/DMX_Materials/MI_Lens.MI_Lens"));
    SurfaceMaterial = Material.Object;
    FixtureMesh = Body.Object;
    LensMesh = Lens.Object;
    LensMaterial = LensSurface.Object;
    WorkLight = CreateDefaultSubobject<UDirectionalLightComponent>(TEXT("WorkLight"));
    WorkLight->SetupAttachment(RootComponent);
    WorkLight->SetMobility(EComponentMobility::Movable);
    WorkLight->SetRelativeRotation(FRotator(-65, -40, 0));
    WorkLight->SetIntensity(16.0f);
    WorkLight->SetCastShadows(false);
    WorkLight->SetVolumetricScatteringIntensity(0);

    auto Fog = CreateDefaultSubobject<UExponentialHeightFogComponent>(TEXT("Haze"));
    Fog->SetupAttachment(RootComponent);
    Fog->SetFogDensity(0.015f);
    Fog->SetFogHeightFalloff(0.01f);
    Fog->SetVolumetricFog(true);
    Fog->SetVolumetricFogDistance(20000);
    Fog->SetVolumetricFogScatteringDistribution(0.2f);
    Fog->SetFogInscatteringColor(FLinearColor::Black);
}
void APreviewSceneActor::BeginPlay()
{
    Super::BeginPlay();
    Bridge = MakeShared<StageMaster::FPreviewBridge>();
    Bridge->OnScene = [this](StageMaster::FScene&& Scene) { ApplyScene(MoveTemp(Scene)); };
    Bridge->OnFrame = [this](StageMaster::FFrame&& Frame) { ApplyFrame(MoveTemp(Frame)); };
    Bridge->OnInvalidated = [this](const FString& Reason) { Invalidate(Reason); };
    Bridge->Start();
}
void APreviewSceneActor::Tick(float DeltaSeconds)
{
    Super::Tick(DeltaSeconds);
    if (Bridge.IsValid()) Bridge->Tick(FPlatformTime::Seconds());
}
void APreviewSceneActor::EndPlay(const EEndPlayReason::Type Reason)
{
    if (Bridge.IsValid()) Bridge->Stop();
    Bridge.Reset();
    Super::EndPlay(Reason);
}
void APreviewSceneActor::ClearVisuals()
{
    for (const auto& Entry : Meshes) Entry.Value->DestroyComponent();
    for (const auto& Entry : Fixtures)
    {
        Entry.Value.Body->DestroyComponent();
        Entry.Value.Light->DestroyComponent();
        Entry.Value.Lens->DestroyComponent();
    }
    Meshes.Reset();
    Fixtures.Reset();
}
void APreviewSceneActor::ApplyScene(StageMaster::FScene&& Scene)
{
    CanEdit = false;
    FrameValid = false;
    ++SceneSerial;
    if (Current.ProjectId != Scene.ProjectId)
    {
        ClearVisuals();
        Current = {};
    }
    TSet<FString> Retained;
    for (const auto& Mesh : Scene.Meshes)
    {
        Retained.Add(Mesh.Id);
        const auto Previous = Current.Meshes.FindByPredicate([&Mesh](const auto& Item) { return Item.Id == Mesh.Id; });
        if (Previous && SameMesh(*Previous, Mesh)) continue;
        auto& Component = Meshes.FindOrAdd(Mesh.Id);
        if (!Component) Component = Attach<UProceduralMeshComponent>(*this);
        Component->SetCollisionEnabled(ECollisionEnabled::QueryOnly);
        Component->SetCollisionResponseToAllChannels(ECR_Block);
        Component->CreateMeshSection_LinearColor(0, Mesh.Vertices, Mesh.Indices, Mesh.Normals,
            TArray<FVector2D>(), TArray<FLinearColor>(), TArray<FProcMeshTangent>(), true);
        auto Material = UMaterialInstanceDynamic::Create(SurfaceMaterial, Component);
        Material->SetVectorParameterValue(TEXT("Color"), Mesh.Color);
        Component->SetMaterial(0, Material);
    }
    for (auto It = Meshes.CreateIterator(); It; ++It)
    {
        if (!Retained.Contains(It.Key())) { It.Value()->DestroyComponent(); It.RemoveCurrent(); }
    }
    Retained.Reset();
    for (const auto& Fixture : Scene.Fixtures)
    {
        Retained.Add(Fixture.Id);
        auto& Visual = Fixtures.FindOrAdd(Fixture.Id);
        if (!Visual.Body)
        {
            Visual.Body = Attach<UStaticMeshComponent>(*this);
            Visual.Body->SetStaticMesh(FixtureMesh);

            Visual.Body->SetCollisionEnabled(ECollisionEnabled::QueryOnly);
            Visual.Body->SetCollisionResponseToAllChannels(ECR_Block);
            Visual.Body->SetCastShadow(false);
            Visual.Lens = Attach<UStaticMeshComponent>(*this);
            Visual.Lens->SetStaticMesh(LensMesh);
            Visual.Lens->SetCollisionEnabled(ECollisionEnabled::NoCollision);
            Visual.Lens->SetCastShadow(false);
            Visual.LensMaterial = UMaterialInstanceDynamic::Create(LensMaterial, Visual.Lens);
            Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Max Light Intensity"), 6000);
            Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Strobe Frequency"), 0);
            Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Strobe Open"), 1);
            Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Gobo Disk Rotation Speed"), 0);
            Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Color Disk Rotation Speed"), 0);
            Visual.Lens->SetMaterial(0, Visual.LensMaterial);
            Visual.Light = Attach<USpotLightComponent>(*this);
            Visual.Light->SetIntensityUnits(ELightUnits::Lumens);
            Visual.Light->SetUseInverseSquaredFalloff(true);
            Visual.Light->SetAttenuationRadius(10000);
            Visual.Light->SetCastShadows(true);
            Visual.Light->SetCastVolumetricShadow(true);
            Visual.Light->SetVolumetricScatteringIntensity(1.0f);
        }
        // Vendor fixed-head mesh has local +Z optics and the lens plane at 14.747 cm.
        const FQuat Rotation = FRotationMatrix::MakeFromZ(Fixture.Direction).ToQuat();
        const FVector BodyOrigin = Fixture.Origin - Fixture.Direction * 14.747;
        Visual.Body->SetWorldLocationAndRotation(BodyOrigin, Rotation);
        Visual.Lens->SetWorldLocationAndRotation(BodyOrigin, Rotation);
        Visual.LensMaterial->SetScalarParameterValue(TEXT("DMX Dimmer"), 0);
        Visual.Light->SetWorldLocation(Fixture.Origin);
        Visual.Light->SetWorldRotation(Fixture.Direction.Rotation());
        Visual.Light->SetOuterConeAngle(Fixture.BeamAngle / 2);
        Visual.Light->SetInnerConeAngle(Fixture.BeamAngle * 0.4f);
        Visual.Light->SetIntensity(0);
    }
    for (auto It = Fixtures.CreateIterator(); It; ++It)
    {
        if (!Retained.Contains(It.Key()))
        {
            It.Value().Body->DestroyComponent();
            It.Value().Light->DestroyComponent();
            It.Value().Lens->DestroyComponent();
            It.RemoveCurrent();
        }
    }
    Current = MoveTemp(Scene);
    ApplyCutaway();
    Status = TEXT("正在同步灯光");
}
void APreviewSceneActor::ApplyFrame(StageMaster::FFrame&& Frame)
{
    CanEdit = Frame.CanEdit;
    FrameValid = Frame.Status != TEXT("unloaded") && Frame.Status != TEXT("missingScene") && Frame.Status != TEXT("stalePlayback");
    Status = StageMaster::StatusLabel(Frame.Status);
    if (Frame.Lights.IsEmpty()) for (const auto& Entry : Fixtures)
    {
        Entry.Value.Light->SetIntensity(0);
        Entry.Value.LensMaterial->SetScalarParameterValue(TEXT("DMX Dimmer"), 0);
    }
    for (const auto& Light : Frame.Lights)
    {
        if (const auto Visual = Fixtures.Find(Light.Id))
        {
            Visual->Light->SetLightColor(Light.Color, false);
            Visual->LensMaterial->SetVectorParameterValue(TEXT("DMX Color"), Light.Color);
            Visual->LensMaterial->SetScalarParameterValue(TEXT("DMX Dimmer"), Light.Intensity);
            // Generic visual scale, never a measured fixture photometry claim.
            Visual->Light->SetIntensity(Light.Intensity * 6000.0f);
        }
    }
}
void APreviewSceneActor::Invalidate(const FString& Reason)
{
    CanEdit = false;
    FrameValid = false;
    Status = Reason;
    for (const auto& Entry : Fixtures)
    {
        Entry.Value.Light->SetIntensity(0);
        Entry.Value.LensMaterial->SetScalarParameterValue(TEXT("DMX Dimmer"), 0);
    }
}
FBox APreviewSceneActor::GetBounds() const
{
    FBox Box(ForceInit);
    for (const auto& Mesh : Current.Meshes) for (const auto& Point : Mesh.Vertices) Box += Point;
    for (const auto& Fixture : Current.Fixtures) Box += Fixture.Origin;
    return Box;
}
FString APreviewSceneActor::FixtureAt(const FHitResult& Hit) const
{
    for (const auto& Entry : Fixtures) if (Hit.GetComponent() == Entry.Value.Body) return Entry.Key;
    return {};
}
const StageMaster::FFixture* APreviewSceneActor::FindFixture(const FString& Id) const
{
    return Current.Fixtures.FindByPredicate([&Id](const auto& Fixture) { return Fixture.Id == Id; });
}
bool APreviewSceneActor::PreviewPosition(const FString& Id, const FVector& Location)
{
    auto Visual = Fixtures.Find(Id);
    if (!FrameValid || !Visual || Location.ContainsNaN() || Location.GetAbsMax() > 10000000) return false;
    const auto Fixture = FindFixture(Id);
    if (!Fixture) return false;
    const FVector BodyOrigin = Location - Fixture->Direction * 14.747;
    Visual->Body->SetWorldLocation(BodyOrigin);
    Visual->Lens->SetWorldLocation(BodyOrigin);
    Visual->Light->SetWorldLocation(Location);
    return true;
}
void APreviewSceneActor::RestorePosition(const FString& Id)
{
    const auto Fixture = FindFixture(Id);
    const auto Visual = Fixtures.Find(Id);
    if (Fixture && Visual)
    {
        const FVector BodyOrigin = Fixture->Origin - Fixture->Direction * 14.747;
        Visual->Body->SetWorldLocation(BodyOrigin);
        Visual->Lens->SetWorldLocation(BodyOrigin);
        Visual->Light->SetWorldLocation(Fixture->Origin);
    }
}
bool APreviewSceneActor::CommitPosition(const FString& Id, const FVector& Location)
{
    const auto Fixture = FindFixture(Id);
    if (!CanEdit || !Fixture || !Bridge.IsValid()) return false;
    const bool Submitted = Bridge->SubmitPlacement(StageMaster::MoveRequest(Current.Stamp, *Fixture, Location));
    CanEdit = false;
    RestorePosition(Id);
    Status = Submitted ? TEXT("正在提交灯位") : TEXT("灯位未提交，请重新启用拖动");
    return Submitted;
}
void APreviewSceneActor::ToggleWorkLight() { WorkLight->SetVisibility(!WorkLight->IsVisible()); }
bool APreviewSceneActor::HasWorkLight() const { return WorkLight->IsVisible(); }

void APreviewSceneActor::ApplyCutaway()
{
    for (const auto& Mesh : Current.Meshes)
    {
        if (auto Component = Meshes.Find(Mesh.Id))
        {
            const bool Hidden = Cutaway && Mesh.EnclosureShell;
            (*Component)->SetCastHiddenShadow(true);
            (*Component)->SetHiddenInGame(Hidden);
            (*Component)->SetCollisionResponseToChannel(ECC_Visibility, Hidden ? ECR_Ignore : ECR_Block);
        }
    }
}
void APreviewSceneActor::ToggleCutaway() { Cutaway = !Cutaway; ApplyCutaway(); }
