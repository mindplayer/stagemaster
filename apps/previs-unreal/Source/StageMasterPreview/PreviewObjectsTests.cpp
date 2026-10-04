#include "PreviewObjects.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"

namespace
{
TSharedPtr<FJsonObject> ParseObjects(const FString& Text)
{
    TSharedPtr<FJsonObject> Result;
    FJsonSerializer::Deserialize(TJsonReaderFactory<>::Create(Text), Result);
    return Result;
}
}
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewObjectSelectionTest, "StageMaster.Previs.ObjectSelection",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewObjectSelectionTest::RunTest(const FString& Parameters)
{
    const TArray<FString> Keys{StageMaster::FixtureKey(TEXT("same")), StageMaster::ConstructionKey(TEXT("same"))};
    const auto Message = MakeShared<FJsonObject>();
    Message->SetArrayField(TEXT("targets"), StageMaster::ObjectValues(Keys));
    TArray<FString> Parsed;
    TestTrue(TEXT("typed identities may have the same raw id"), StageMaster::ReadObjectSelection(Message, Parsed));
    TestTrue(TEXT("whole ordered group retained"), Parsed == Keys);
    for (const TCHAR* Bad : {TEXT(R"({"targets":[{"kind":"space","id":"room"}]})"),
        TEXT(R"({"targets":[{"kind":"construction","id":"rig","extra":true}]})"),
        TEXT(R"({"targets":[{"kind":"construction","id":""}]})"),
        TEXT(R"({"targets":[{"kind":"construction","id":"rig"},{"kind":"construction","id":"rig"}]})")})
    {
        TestFalse(TEXT("invalid group rejected"), StageMaster::ReadObjectSelection(ParseObjects(Bad), Parsed));
        TestTrue(TEXT("rejected group leaves previous state"), Parsed == Keys);
    }
    TArray<FString> Large;
    for (int32 Index = 0; Index < 1024; ++Index) Large.Add(StageMaster::ConstructionKey(FString::FromInt(Index)));
    Message->SetArrayField(TEXT("targets"), StageMaster::ObjectValues(Large));
    TestTrue(TEXT("maximum view selection"), StageMaster::ReadObjectSelection(Message, Parsed));
    Large.Add(StageMaster::FixtureKey(TEXT("extra")));
    Message->SetArrayField(TEXT("targets"), StageMaster::ObjectValues(Large));
    TestFalse(TEXT("view overflow"), StageMaster::ReadObjectSelection(Message, Parsed));
    TestEqual(TEXT("overflow preserves accepted count"), Parsed.Num(), 1024);
    Message->SetArrayField(TEXT("targets"), {});
    TestTrue(TEXT("clear view selection"), StageMaster::ReadObjectSelection(Message, Parsed));
    TestTrue(TEXT("clear is empty"), Parsed.IsEmpty());

    const StageMaster::FStamp Stamp{TEXT("bridge"), TEXT("9007199254740993"), 7};
    const auto Request = StageMaster::ObjectTranslationRequest(Stamp, Keys, FVector(125, -200, -25));
    if (!TestTrue(TEXT("mixed move proposal"), Request.IsValid())) return false;
    TestFalse(TEXT("no partial fixture-only identity"), Request->HasField(TEXT("fixtureIds")));
    TestEqual(TEXT("exact revision"), Request->GetStringField(TEXT("version")), Stamp.Version);
    TestTrue(TEXT("wire round trip"), StageMaster::ReadObjectSelection(Request, Parsed) && Parsed == Keys);
    TestEqual(TEXT("metres"), Request->GetObjectField(TEXT("deltaMeters"))->GetStringField(TEXT("x")), FString(TEXT("1.25")));
    TestEqual(TEXT("handedness"), Request->GetObjectField(TEXT("deltaMeters"))->GetStringField(TEXT("y")), FString(TEXT("2")));
    TestEqual(TEXT("height"), Request->GetObjectField(TEXT("deltaMeters"))->GetStringField(TEXT("z")), FString(TEXT("-0.25")));
    TestFalse(TEXT("unknown key cannot become empty selection"), StageMaster::ObjectTranslationRequest(Stamp, {TEXT("space:room")}, FVector::ZeroVector).IsValid());
    TestFalse(TEXT("duplicate move rejected"), StageMaster::ObjectTranslationRequest(Stamp, {Keys[0], Keys[0]}, FVector::ZeroVector).IsValid());
    TestFalse(TEXT("empty move rejected"), StageMaster::ObjectTranslationRequest(Stamp, {}, FVector::ZeroVector).IsValid());
    Large.SetNum(256);
    TestTrue(TEXT("256 direct movers"), StageMaster::ObjectTranslationRequest(Stamp, Large, FVector::ZeroVector).IsValid());
    Large.Add(Keys[0]);
    TestFalse(TEXT("257 direct movers rejected"), StageMaster::ObjectTranslationRequest(Stamp, Large, FVector::ZeroVector).IsValid());
    TestFalse(TEXT("coordinate bound"), StageMaster::ObjectTranslationRequest(Stamp, Keys, FVector(20000001, 0, 0)).IsValid());
    return true;
}
IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewObjectOwnersTest, "StageMaster.Previs.ObjectOwners",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewObjectOwnersTest::RunTest(const FString& Parameters)
{
    auto Input = ParseObjects(TEXT(R"({"protocol":2,"bridgeId":"b","generation":1,"version":"1","scene":{"projectId":"p","projectName":"场地","meshes":[
        {"id":"rig","name":"桁架","constructionId":"rig-owner","movable":true,"attachedFixtureIds":["lamp"],"color":[0.2,0.3,0.4],"triangles":[[[0,0,1],[2,0,1],[0,3,1]]]}
        ],"fixtures":[{"id":"lamp","name":"灯","originMeters":[1,2,3],"direction":[0,0,-1],"fullBeamAngleDegrees":25,"optics":"generic-illustrative","placement":{"fixtureId":"lamp","spaceId":null,"positionMeters":{"x":"1","y":"2","z":"3"},"rotationDegreesXYZ":{"x":"0","y":"0","z":"0"}}}]}})"));
    StageMaster::FScene Scene;
    FString Error;
    TestTrue(TEXT("owned projection accepted"), StageMaster::ReadScene(Input, Scene, Error));
    if (Scene.Meshes.IsEmpty()) return false;
    TestEqual(TEXT("business owner differs from render mesh"), Scene.Meshes[0].ConstructionId, FString(TEXT("rig-owner")));
    TestTrue(TEXT("attachment retained"), Scene.Meshes[0].AttachedFixtureIds == TArray<FString>{TEXT("lamp")});
    const auto Data = Input->GetObjectField(TEXT("scene"));
    const auto First = Data->GetArrayField(TEXT("meshes"))[0]->AsObject();
    const TSharedPtr<FJsonObject> Second = MakeShared<FJsonObject>(*First);
    Second->SetStringField(TEXT("id"), TEXT("second-mesh"));
    Data->SetArrayField(TEXT("meshes"), {MakeShared<FJsonValueObject>(First), MakeShared<FJsonValueObject>(Second)});
    TestTrue(TEXT("multiple meshes may share an owner"), StageMaster::ReadScene(Input, Scene, Error));
    Second->SetBoolField(TEXT("movable"), false);
    TestFalse(TEXT("inconsistent or immovable attached owner rejected"), StageMaster::ReadScene(Input, Scene, Error));
    TestTrue(TEXT("bad snapshot did not change accepted owners"), Scene.Meshes.Num() == 2 && Scene.Meshes[1].Movable);
    Second->SetBoolField(TEXT("movable"), true);
    Second->SetStringField(TEXT("constructionId"), TEXT("other-owner"));
    TestFalse(TEXT("one fixture cannot belong to two owners"), StageMaster::ReadScene(Input, Scene, Error));
    Second->SetStringField(TEXT("constructionId"), TEXT("rig-owner"));
    for (const auto& Object : {First, Second}) Object->SetArrayField(TEXT("attachedFixtureIds"), {MakeShared<FJsonValueString>(TEXT("missing"))});
    TestFalse(TEXT("missing mounted lamp rejects scene"), StageMaster::ReadScene(Input, Scene, Error));
    for (const auto& Object : {First, Second})
    {
        Object->SetArrayField(TEXT("attachedFixtureIds"), {});
        Object->SetBoolField(TEXT("movable"), false);
    }
    TestTrue(TEXT("immovable multi-mesh enclosure"), StageMaster::ReadScene(Input, Scene, Error));
    for (const auto& Object : {First, Second}) Object->RemoveField(TEXT("constructionId"));
    TestFalse(TEXT("partial new owner contract rejected"), StageMaster::ReadScene(Input, Scene, Error));
    for (const auto& Object : {First, Second}) { Object->RemoveField(TEXT("movable")); Object->RemoveField(TEXT("attachedFixtureIds")); }
    TestTrue(TEXT("legacy render-only meshes still accepted"), StageMaster::ReadScene(Input, Scene, Error));
    TestTrue(TEXT("legacy mesh has no editable owner"), Scene.Meshes[0].ConstructionId.IsEmpty() && !Scene.Meshes[0].Movable);
    return true;
}
#endif
