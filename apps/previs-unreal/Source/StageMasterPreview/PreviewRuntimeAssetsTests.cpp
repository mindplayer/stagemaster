#if WITH_DEV_AUTOMATION_TESTS
#include "CoreMinimal.h"
#include "Engine/StaticMesh.h"
#include "HAL/IConsoleManager.h"
#include "Materials/MaterialInterface.h"
#include "Misc/AutomationTest.h"

// Exactly the assets used by PreviewSceneActor, not the broader editor reference library.
// Run this from the cooked Game as well: editor loading cannot prove a packaged dependency.
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewRuntimeAssetsTest, "StageMaster.Previs.RuntimeAssets",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::ClientContext | EAutomationTestFlags::EngineFilter)
bool FPreviewRuntimeAssetsTest::RunTest(const FString& Parameters)
{
    const auto Codec = IConsoleManager::Get().FindConsoleVariable(TEXT("PixelStreaming2.Encoder.Codec"));
    if (TestNotNull(TEXT("official encoder codec setting"), Codec))
        TestEqual(TEXT("packaged and editor default codec stays H264"), Codec->GetString(), FString(TEXT("H264")));
    for (const TCHAR* Path : {
        TEXT("/DMXFixtures/LightFixtures/Meshes/SM_Static_Base.SM_Static_Base"),
        TEXT("/DMXFixtures/LightFixtures/Meshes/SM_Static_Lens.SM_Static_Lens"),
        TEXT("/Engine/BasicShapes/Cube.Cube")})
    {
        const auto Mesh = LoadObject<UStaticMesh>(nullptr, Path);
        if (TestNotNull(Path, Mesh)) TestTrue(TEXT("nonempty visual mesh"), Mesh->GetBoundingBox().IsValid != 0);
    }
    for (const TCHAR* Path : {
        TEXT("/DMXFixtures/LightFixtures/DMX_Materials/MI_Lens.MI_Lens"),
        TEXT("/Engine/BasicShapes/BasicShapeMaterial.BasicShapeMaterial")})
        TestNotNull(Path, LoadObject<UMaterialInterface>(nullptr, Path));
    return true;
}
#endif
