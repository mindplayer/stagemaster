#include "PreviewInput.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewMouseReleaseTest, "StageMaster.Previs.MouseReleaseEndpoint",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewMouseReleaseTest::RunTest(const FString& Parameters)
{
    // Prefix represents another protocol field already consumed by the SDK.
    TArray<uint8> Payload{99, 0, 255, 255, 0, 128};
    FMemoryReader Reader(Payload);
    Reader.Seek(1);
    uint8 Button = 3;
    bool InRange = false;
    FVector2D Position(-1, -1);
    TestTrue(TEXT("final coordinates available without a preceding move"),
        StageMaster::ReadMouseRelease(Reader, 1920, 1080, Button, Position, InRange));
    TestTrue(TEXT("ordinary coordinate is inside"), InRange);
    TestEqual(TEXT("left release"), Button, uint8(0));
    TestEqual(TEXT("endpoint uses viewport dimensions"), Position.X, 1920.0);
    TestTrue(TEXT("endpoint preserves subpixel precision"), FMath::IsNearlyEqual(Position.Y, 32768.0 * 1080 / 65535, 0.000001));
    TestEqual(TEXT("official release handler reads the original payload"), Reader.Tell(), int64(1));
    TestFalse(TEXT("zero sized viewport rejected"), StageMaster::ReadMouseRelease(Reader, 0, 1080, Button, Position, InRange));
    Payload[4] = 255;
    Payload[5] = 255;
    TestTrue(TEXT("outside release still needs SDK capture cleanup"), StageMaster::ReadMouseRelease(Reader, 1920, 1080, Button, Position, InRange));
    TestFalse(TEXT("outside sentinel never moves a fixture to the viewport corner"), InRange);
    Payload[1] = 5;
    TestFalse(TEXT("unknown button rejected"), StageMaster::ReadMouseRelease(Reader, 1920, 1080, Button, Position, InRange));
    TestEqual(TEXT("failed parse preserves button"), Button, uint8(0));
    for (int32 Length : {0, 1, 4, 6})
    {
        TArray<uint8> Invalid;
        Invalid.SetNumZeroed(Length);
        FMemoryReader Bad(Invalid);
        TestFalse(TEXT("truncated or extended payload rejected"), StageMaster::ReadMouseRelease(Bad, 1920, 1080, Button, Position, InRange));
        TestEqual(TEXT("invalid payload not consumed"), Bad.Tell(), int64(0));
    }
    return true;
}
#endif
