#include "PreviewProtocol.h"

#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"
#include "KismetProceduralMeshLibrary.h"
#include "ProceduralMeshComponent.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"

namespace
{
TSharedPtr<FJsonObject> Parse(const FString& Text)
{
    TSharedPtr<FJsonObject> Result;
    FJsonSerializer::Deserialize(TJsonReaderFactory<>::Create(Text), Result);
    return Result;
}
const TCHAR* SceneJson = TEXT(R"json({
    "protocol":1,"bridgeId":"test-bridge","generation":7,"version":"9007199254740993",
    "scene":{"projectId":"project","projectName":"预演测试","meshes":[
        {"id":"floor","name":"地面","color":[0.2,0.3,0.4],"triangles":[[[0,0,1],[2,0,1],[0,3,1]]]}
    ],"fixtures":[
        {"id":"fixture","name":"灯具","originMeters":[2,3,4],"direction":[0,0,-1],
         "fullBeamAngleDegrees":25,"optics":"generic-illustrative",
         "placement":{"fixtureId":"fixture","spaceId":"room","positionMeters":{"x":"2","y":"3","z":"4"},
                      "rotationDegreesXYZ":{"x":"25","y":"0","z":"0"}}}
    ]}
})json");
}

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewProjectionTest, "StageMaster.Previs.ProjectionBoundary",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewProjectionTest::RunTest(const FString& Parameters)
{
    StageMaster::FScene Scene;
    FString Error;
    TestTrue(TEXT("read validated scene"), StageMaster::ReadScene(Parse(SceneJson), Scene, Error));
    if (Scene.Meshes.Num() != 1 || Scene.Fixtures.Num() != 1) return false;
    TestEqual(TEXT("64 bit version remains a string"), Scene.Stamp.Version, FString(TEXT("9007199254740993")));
    TestTrue(TEXT("metres and handedness"), Scene.Fixtures[0].Origin.Equals(FVector(200, -300, 400)));
    TestTrue(TEXT("downward installation ray"), Scene.Fixtures[0].Direction.Equals(FVector(0, 0, -1)));
    TestTrue(TEXT("reflected triangle preserves upward normal"), Scene.Meshes[0].Normals[0].Equals(FVector::UpVector));
    TestTrue(TEXT("reflected triangle preserves dimensions"), Scene.Meshes[0].Vertices[2].Equals(FVector(0, -300, 100)));
    TArray<FVector> ReferenceVertices, ReferenceNormals;
    TArray<int32> ReferenceIndices;
    TArray<FVector2D> ReferenceUvs;
    TArray<FProcMeshTangent> ReferenceTangents;
    UKismetProceduralMeshLibrary::GenerateBoxMesh(FVector(1), ReferenceVertices, ReferenceIndices,
        ReferenceNormals, ReferenceUvs, ReferenceTangents);
    auto Facing = [](const TArray<FVector>& Vertices, const TArray<int32>& Indices, const FVector& Normal)
    {
        return FVector::DotProduct(FVector::CrossProduct(Vertices[Indices[1]] - Vertices[Indices[0]],
            Vertices[Indices[2]] - Vertices[Indices[0]]), Normal);
    };
    TestTrue(TEXT("front face agrees with UE's own box generator"),
        Facing(ReferenceVertices, ReferenceIndices, ReferenceNormals[0]) *
        Facing(Scene.Meshes[0].Vertices, Scene.Meshes[0].Indices, Scene.Meshes[0].Normals[0]) > 0);
    TestTrue(TEXT("coordinate round trip"), StageMaster::ToMeters(StageMaster::ToUnreal(FVector(-1.2, 4.5, 6.7))).Equals(FVector(-1.2, 4.5, 6.7)));

    const auto Request = StageMaster::MoveRequest(Scene.Stamp, Scene.Fixtures[0], FVector(-12.5, 0, 400));
    if (!TestTrue(TEXT("move request exists"), Request.IsValid())) return false;
    const auto Placement = Request->GetObjectField(TEXT("placement"));
    const auto Position = Placement->GetObjectField(TEXT("positionMeters"));
    TestEqual(TEXT("negative fraction remains negative"), Position->GetStringField(TEXT("x")), FString(TEXT("-0.125")));
    TestEqual(TEXT("negative zero becomes canonical zero"), Position->GetStringField(TEXT("y")), FString(TEXT("0")));
    TestEqual(TEXT("room membership preserved"), Placement->GetStringField(TEXT("spaceId")), FString(TEXT("room")));
    TestEqual(TEXT("installation rotation preserved"), Placement->GetObjectField(TEXT("rotationDegreesXYZ"))->GetStringField(TEXT("x")), FString(TEXT("25")));
    TestEqual(TEXT("move does not mutate snapshot"), Scene.Fixtures[0].Placement->GetObjectField(TEXT("positionMeters"))->GetStringField(TEXT("x")), FString(TEXT("2")));
    TestFalse(TEXT("out of range move rejected"), StageMaster::MoveRequest(Scene.Stamp, Scene.Fixtures[0], FVector(10000100, 0, 0)).IsValid());
    return true;
}

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewValidationTest, "StageMaster.Previs.InvalidResponses",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewValidationTest::RunTest(const FString& Parameters)
{
    StageMaster::FStamp Stamp;
    auto Object = Parse(SceneJson);
    for (const TCHAR* Version : {TEXT("01"), TEXT("-1"), TEXT("1.0"), TEXT("18446744073709551616")})
    {
        Object->SetStringField(TEXT("version"), Version);
        TestFalse(TEXT("invalid version rejected"), StageMaster::ReadStamp(Object, Stamp));
    }
    Object->SetStringField(TEXT("version"), TEXT("18446744073709551615"));
    TestTrue(TEXT("u64 maximum preserved"), StageMaster::ReadStamp(Object, Stamp));
    Object->SetNumberField(TEXT("generation"), 0.5);
    TestFalse(TEXT("fractional generation rejected"), StageMaster::ReadStamp(Object, Stamp));
    Object->SetNumberField(TEXT("generation"), 4294967296.0);
    TestFalse(TEXT("generation overflow rejected"), StageMaster::ReadStamp(Object, Stamp));

    StageMaster::FScene Scene;
    Scene.Name = TEXT("keep previous snapshot");
    FString Error;
    TestFalse(TEXT("malformed scene rejected"), StageMaster::ReadScene(Object, Scene, Error));
    TestEqual(TEXT("failed parse is atomic"), Scene.Name, FString(TEXT("keep previous snapshot")));
    Object = Parse(SceneJson);
    auto SceneObject = Object->GetObjectField(TEXT("scene"));
    auto Fixtures = SceneObject->GetArrayField(TEXT("fixtures"));
    const auto Duplicate = Fixtures[0];
    Fixtures.Add(Duplicate);
    SceneObject->SetArrayField(TEXT("fixtures"), Fixtures);
    TestFalse(TEXT("duplicate fixture rejected"), StageMaster::ReadScene(Object, Scene, Error));

    Object = Parse(SceneJson);
    auto MeshObject = Object->GetObjectField(TEXT("scene"))->GetArrayField(TEXT("meshes"))[0]->AsObject();
    TestTrue(TEXT("legacy mesh accepted"), StageMaster::ReadScene(Object, Scene, Error));
    TestFalse(TEXT("legacy mesh stays solid"), Scene.Meshes[0].EnclosureShell);
    MeshObject->SetStringField(TEXT("viewRole"), TEXT("enclosureShell"));
    TestTrue(TEXT("shell display role accepted"), StageMaster::ReadScene(Object, Scene, Error));
    TestTrue(TEXT("shell display role retained"), Scene.Meshes[0].EnclosureShell);
    MeshObject->SetStringField(TEXT("viewRole"), TEXT("unknown"));
    TestFalse(TEXT("unknown display role rejected"), StageMaster::ReadScene(Object, Scene, Error));
    TestTrue(TEXT("bad role retains prior scene"), Scene.Meshes[0].EnclosureShell);

    StageMaster::FFrame Frame;
    Object = Parse(TEXT(R"json({"protocol":1,"bridgeId":"bridge","generation":0,"version":"2","status":"running",
        "source":{"kind":"playback"},"canEdit":false,"lights":[{"fixtureId":"fixture","intensity":0.5,"color":[1,0.25,0]}]})json"));
    TestTrue(TEXT("playback frame accepted"), StageMaster::ReadFrame(Object, Frame, Error));
    Object->GetArrayField(TEXT("lights"))[0]->AsObject()->SetNumberField(TEXT("intensity"), 1.1);
    TestFalse(TEXT("unbounded brightness rejected"), StageMaster::ReadFrame(Object, Frame, Error));
    TestEqual(TEXT("bad frame retains previous light values"), Frame.Lights[0].Intensity, 0.5f);
    return true;
}
#endif
