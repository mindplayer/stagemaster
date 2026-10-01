#include "PreviewSceneActor.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Components/StaticMeshComponent.h"
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewFixturePickingTest, "StageMaster.Previs.FixturePicking",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewFixturePickingTest::RunTest(const FString& Parameters)
{
    FPreviewFixtureVisual Fixed;
    Fixed.Body = NewObject<UStaticMeshComponent>();
    TestFalse(TEXT("empty space cannot hit absent arms of a fixed fixture"), Fixed.ContainsHitComponent(FHitResult().GetComponent()));
    TestTrue(TEXT("fixed fixture body remains selectable"), Fixed.ContainsHitComponent(Fixed.Body));
    TestFalse(TEXT("other scenery cannot select this fixture"), Fixed.ContainsHitComponent(NewObject<UStaticMeshComponent>()));

    FPreviewFixtureVisual Moving;
    Moving.Body = NewObject<UStaticMeshComponent>();
    Moving.Base = NewObject<UStaticMeshComponent>();
    Moving.ArmLeft = NewObject<UStaticMeshComponent>();
    Moving.ArmRight = NewObject<UStaticMeshComponent>();
    for (auto Part : {Moving.Body, Moving.Base, Moving.ArmLeft, Moving.ArmRight})
        TestTrue(TEXT("all moving head structural parts remain selectable"), Moving.ContainsHitComponent(Part));
    TestFalse(TEXT("missed ray cannot select a moving head"), Moving.ContainsHitComponent(nullptr));
    TestFalse(TEXT("another fixture cannot steal this hit"), Moving.ContainsHitComponent(Fixed.Body));
    return true;
}
#endif
