#include "PreviewSelection.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewGroupMovementTest, "StageMaster.Previs.GroupMovement",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewGroupMovementTest::RunTest(const FString& Parameters)
{
    const TArray<FString> Group{TEXT("a"), TEXT("b"), TEXT("c")};
    TestTrue(TEXT("dragging any member keeps order and active fixture"), StageMaster::SelectFixture(Group, TEXT("b"), false, true) == Group);
    TestTrue(TEXT("view click replaces group"), StageMaster::SelectFixture(Group, TEXT("b"), false, false) == TArray<FString>{TEXT("b")});
    TestTrue(TEXT("shift removes one member"), StageMaster::SelectFixture(Group, TEXT("b"), true, true) == TArray<FString>({TEXT("a"), TEXT("c")}));
    TestTrue(TEXT("new shift selection becomes active"), StageMaster::SelectFixture(Group, TEXT("d"), true, false).Last() == TEXT("d"));
    TestTrue(TEXT("plain blank clears"), StageMaster::SelectFixture(Group, TEXT(""), false, true).IsEmpty());
    TestTrue(TEXT("shift blank preserves"), StageMaster::SelectFixture(Group, TEXT(""), true, false) == Group);
    const auto Message = MakeShared<FJsonObject>();
    TArray<TSharedPtr<FJsonValue>> Values;
    for (int32 Index = 0; Index < 1024; ++Index) Values.Add(MakeShared<FJsonValueString>(FString::FromInt(Index)));
    Message->SetArrayField(TEXT("fixtureIds"), Values);
    TArray<FString> Parsed;
    TestTrue(TEXT("full stage selection fits bounded decoder"), StageMaster::ReadFixtureSelection(Message, Parsed));
    TestEqual(TEXT("all identities retained"), Parsed.Num(), 1024);
    const auto Duplicate = Values[0];
    Values.Add(Duplicate);
    Message->SetArrayField(TEXT("fixtureIds"), Values);
    TestFalse(TEXT("selection overflow rejected"), StageMaster::ReadFixtureSelection(Message, Parsed));
    TestEqual(TEXT("failed decode leaves prior selection"), Parsed.Num(), 1024);
    Message->SetArrayField(TEXT("fixtureIds"), {Values[0], Values[0]});
    TestFalse(TEXT("duplicates rejected"), StageMaster::ReadFixtureSelection(Message, Parsed));
    Message->SetArrayField(TEXT("fixtureIds"), {});
    TestTrue(TEXT("clear selection accepted"), StageMaster::ReadFixtureSelection(Message, Parsed));
    TestTrue(TEXT("selection cleared"), Parsed.IsEmpty());

    StageMaster::FStamp Stamp{TEXT("bridge"), TEXT("9007199254740993"), 7};
    auto Move = StageMaster::TranslationRequest(Stamp, Group, FVector(125, -200, -62.5));
    TestTrue(TEXT("group translation encoded"), Move.IsValid());
    if (!Move.IsValid()) return false;
    TestEqual(TEXT("revision never rounded through a double"), Move->GetStringField(TEXT("version")), Stamp.Version);
    const auto Delta = Move->GetObjectField(TEXT("deltaMeters"));
    TestEqual(TEXT("centimetres converted to metres"), Delta->GetStringField(TEXT("x")), FString(TEXT("1.25")));
    TestEqual(TEXT("coordinate handedness converted once"), Delta->GetStringField(TEXT("y")), FString(TEXT("2")));
    TestEqual(TEXT("negative height preserved"), Delta->GetStringField(TEXT("z")), FString(TEXT("-0.625")));
    TestFalse(TEXT("empty group rejected"), StageMaster::TranslationRequest(Stamp, {}, FVector::ZeroVector).IsValid());
    TestFalse(TEXT("duplicate movement rejected"), StageMaster::TranslationRequest(Stamp, {TEXT("a"), TEXT("a")}, FVector::ZeroVector).IsValid());
    TArray<FString> Large;
    for (int32 Index = 0; Index < 256; ++Index) Large.Add(FString::FromInt(Index));
    TestTrue(TEXT("256 movers accepted"), StageMaster::TranslationRequest(Stamp, Large, FVector(20000000, 0, 0)).IsValid());
    Large.Add(TEXT("extra"));
    TestFalse(TEXT("257 movers rejected"), StageMaster::TranslationRequest(Stamp, Large, FVector::ZeroVector).IsValid());
    TestFalse(TEXT("delta overflow rejected"), StageMaster::TranslationRequest(Stamp, Group, FVector(20000001, 0, 0)).IsValid());
    Move = StageMaster::TranslationRequest(Stamp, Group, FVector(0, 0.0000001, 0));
    TestEqual(TEXT("negative zero normalized"), Move->GetObjectField(TEXT("deltaMeters"))->GetStringField(TEXT("y")), FString(TEXT("0")));

    FVector Point;
    TestTrue(TEXT("horizontal plane"), StageMaster::DragPlanePoint(FVector(10, 20, 100), FVector(0, 0, -1), FVector(0, 0, 25), FVector::UpVector, Point));
    TestTrue(TEXT("horizontal hit maintains height"), Point.Equals(FVector(10, 20, 25)));
    TestTrue(TEXT("vertical plane"), StageMaster::DragPlanePoint(FVector(-100, 20, 80), FVector(1, 0, -0.2), FVector::ZeroVector, FVector::ForwardVector, Point));
    TestTrue(TEXT("vertical hit preserves ray geometry"), Point.Equals(FVector(0, 20, 60)));
    const FVector Before = Point;
    TestFalse(TEXT("parallel view rejects drag"), StageMaster::DragPlanePoint(FVector::ZeroVector, FVector::UpVector, FVector::ZeroVector, FVector::ForwardVector, Point));
    TestTrue(TEXT("failed projection leaves old point"), Point == Before);
    TestFalse(TEXT("plane behind camera rejected"), StageMaster::DragPlanePoint(FVector(0, 0, 10), FVector::UpVector, FVector::ZeroVector, FVector::UpVector, Point));
    return true;
}
#endif
