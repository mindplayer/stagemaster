#include "PreviewTransform.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewTransformTest, "StageMaster.Previs.GroupTransform",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewTransformTest::RunTest(const FString& Parameters)
{
    const FVector Center = StageMaster::ToUnreal(FVector(1, 2.125, 4));
    const FVector Start = StageMaster::ToUnreal(FVector(0, 2.125, 3));
    const auto Next = StageMaster::TransformPoint(Start, Center, 90, 2);
    TestTrue(TEXT("matches independent project-world expected rotation and scale"),
        StageMaster::ToMeters(Next).Equals(FVector(1, 0.125, 2), 1e-9));
    TestTrue(TEXT("inverse recovers source"), StageMaster::TransformPoint(Next, Center, -90, 0.5).Equals(Start, 1e-8));
    TestTrue(TEXT("pivot stays fixed even for single fixture"), StageMaster::TransformPoint(Center, Center, 37, 20).Equals(Center));
    const StageMaster::FStamp Stamp{TEXT("bridge"), TEXT("9007199254740993"), 7};
    const auto Request = StageMaster::TransformRequest(Stamp, {TEXT("a"), TEXT("b")}, -90.125, 1.25);
    TestTrue(TEXT("valid group request"), Request.IsValid());
    if (!Request.IsValid()) return false;
    TestEqual(TEXT("exact version preserved"), Request->GetStringField(TEXT("version")), Stamp.Version);
    TestEqual(TEXT("angle encoded without coordinate handedness loss"), Request->GetStringField(TEXT("yawDegrees")), FString(TEXT("-90.125")));
    TestFalse(TEXT("does not carry stale translation"), Request->HasField(TEXT("deltaMeters")));
    double Yaw = 0, Scale = 0;
    TestTrue(TEXT("read exact proposal"), StageMaster::ReadTransform(Request, Yaw, Scale));
    TestEqual(TEXT("scale decoded"), Scale, 1.25);
    for (const FString Invalid : {TEXT("0"), TEXT("-1"), TEXT("100.1"), TEXT("NaN"), TEXT("1e1"), TEXT("1.0000001"), TEXT("1.")})
    {
        Request->SetStringField(TEXT("spacingScale"), Invalid);
        TestFalse(TEXT("invalid precise scale rejected"), StageMaster::ReadTransform(Request, Yaw, Scale));
    }
    TestFalse(TEXT("duplicate targets rejected"), StageMaster::TransformRequest(Stamp, {TEXT("a"),TEXT("a")}, 90, 1).IsValid());
    TestFalse(TEXT("negative scale rejected"), StageMaster::TransformRequest(Stamp, {TEXT("a")}, 90, -1).IsValid());
    TestFalse(TEXT("out of range angle rejected"), StageMaster::TransformRequest(Stamp, {TEXT("a")}, 361, 1).IsValid());
    return true;
}
#endif
