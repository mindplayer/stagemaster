#if WITH_DEV_AUTOMATION_TESTS
#include "CoreMinimal.h"
#include "Engine/StaticMesh.h"
#include "GameFramework/Actor.h"
#include "Materials/MaterialInterface.h"
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewLibraryTest, "StageMaster.Previs.OfficialVisualLibrary",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewLibraryTest::RunTest(const FString& Parameters)
{
    // Loading vendor assets does not create actors or output DMX.
    for (const TCHAR* Name : {TEXT("BP_StaticHead"), TEXT("BP_MovingHead"), TEXT("BP_WashLED"),
        TEXT("BP_StaticMatrix"), TEXT("BP_MovingMatrix"), TEXT("BP_StaticStrobe"), TEXT("BP_MovingMiror")})
    {
        const FString Path = FString::Printf(TEXT("/DMXFixtures/LightFixtures/%s.%s_C"), Name, Name);
        TestNotNull(*FString::Printf(TEXT("official fixture class %s"), Name), LoadClass<AActor>(nullptr, *Path));
    }
    for (const TCHAR* Path : {
        TEXT("/DMXFixtures/Laser/BP_LaserModule.BP_LaserModule_C"),
        TEXT("/DMXFixtures/Pyro/BP_PyroModule.BP_PyroModule_C"),
        TEXT("/DMXFixtures/WaterFountains/BP_WaterSource.BP_WaterSource_C"),
        TEXT("/DMXFixtures/Fireworks/BP_FireWorksLauncher.BP_FireWorksLauncher_C")})
        TestNotNull(TEXT("official auxiliary visual class"), LoadClass<AActor>(nullptr, Path));
    for (const TCHAR* Name : {TEXT("MI_Beam"), TEXT("MI_Lens"), TEXT("MI_LightNoGobo"), TEXT("MI_MatrixBeam")})
    {
        const FString Path = FString::Printf(TEXT("/DMXFixtures/LightFixtures/DMX_Materials/%s.%s"), Name, Name);
        auto Material = LoadObject<UMaterialInterface>(nullptr, *Path);
        if (!TestNotNull(*FString::Printf(TEXT("official optical material %s"), Name), Material)) continue;
        TArray<FMaterialParameterInfo> Infos;
        TArray<FGuid> Ids;
        Material->GetAllScalarParameterInfo(Infos, Ids);
        for (const auto& Info : Infos) AddInfo(FString::Printf(TEXT("%s scalar: %s"), Name, *Info.Name.ToString()));
        Infos.Reset(); Ids.Reset();
        Material->GetAllVectorParameterInfo(Infos, Ids);
        for (const auto& Info : Infos) AddInfo(FString::Printf(TEXT("%s vector: %s"), Name, *Info.Name.ToString()));
    }
    for (const TCHAR* Name : {TEXT("SM_Static_Base"), TEXT("SM_Static_Lens"), TEXT("SM_Beam_RM")})
    {
        const FString Path = FString::Printf(TEXT("/DMXFixtures/LightFixtures/Meshes/%s.%s"), Name, Name);
        const auto Mesh = LoadObject<UStaticMesh>(nullptr, *Path);
        if (TestNotNull(TEXT("official fixed fixture mesh"), Mesh))
            AddInfo(FString::Printf(TEXT("%s bounds: %s"), Name, *Mesh->GetBoundingBox().ToString()));
    }
    return true;
}
#endif
