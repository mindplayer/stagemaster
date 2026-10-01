#include "PreviewFrameFreshness.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewBusyFrameTest, "StageMaster.Previs.BoundedBusyFrame",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewBusyFrameTest::RunTest(const FString& Parameters)
{
    using StageMaster::HoldBusyFrame;
    TestTrue(TEXT("short local contention retains a valid frame"), HoldBusyFrame(503, true, true, 10, 10.03));
    TestTrue(TEXT("retries use the original valid timestamp"), HoldBusyFrame(503, true, true, 10, 11.999));
    TestFalse(TEXT("two seconds expires even with repeated busy responses"), HoldBusyFrame(503, true, true, 10, 12));
    TestFalse(TEXT("never hold before the first valid frame"), HoldBusyFrame(503, true, false, 10, 10.03));
    TestFalse(TEXT("scene or placement failures are not masked"), HoldBusyFrame(503, false, true, 10, 10.03));
    TestFalse(TEXT("authorization failure invalidates immediately"), HoldBusyFrame(403, true, true, 10, 10.03));
    TestFalse(TEXT("changed scene revision invalidates immediately"), HoldBusyFrame(409, true, true, 10, 10.03));
    TestFalse(TEXT("invalid payload invalidates immediately"), HoldBusyFrame(422, true, true, 10, 10.03));
    TestFalse(TEXT("backwards clock cannot renew a frame"), HoldBusyFrame(503, true, true, 10, 9));
    return true;
}
#endif
