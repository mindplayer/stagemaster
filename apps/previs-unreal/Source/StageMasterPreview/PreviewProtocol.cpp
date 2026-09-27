#include "PreviewProtocol.h"
#include "Dom/JsonValue.h"

namespace StageMaster
{
namespace
{
bool Text(const TSharedPtr<FJsonObject>& Object, const TCHAR* Key, FString& Out)
{
    return Object.IsValid() && Object->TryGetStringField(Key, Out) && !Out.IsEmpty() && Out.Len() <= 4096;
}
bool Array(const TSharedPtr<FJsonObject>& Object, const TCHAR* Key, const TArray<TSharedPtr<FJsonValue>>*& Out, int32 Maximum)
{
    return Object.IsValid() && Object->TryGetArrayField(Key, Out) && Out->Num() <= Maximum;
}
bool ObjectValue(const TSharedPtr<FJsonValue>& Value, TSharedPtr<FJsonObject>& Out)
{
    const TSharedPtr<FJsonObject>* Found = nullptr;
    if (!Value.IsValid() || !Value->TryGetObject(Found) || !Found || !Found->IsValid()) return false;
    Out = *Found;
    return true;
}
bool ObjectField(const TSharedPtr<FJsonObject>& Parent, const TCHAR* Key, TSharedPtr<FJsonObject>& Out)
{
    const TSharedPtr<FJsonObject>* Found = nullptr;
    if (!Parent.IsValid() || !Parent->TryGetObjectField(Key, Found) || !Found || !Found->IsValid()) return false;
    Out = *Found;
    return true;
}
bool Triple(const TSharedPtr<FJsonValue>& Value, FVector& Out, double Maximum)
{
    const TArray<TSharedPtr<FJsonValue>>* Values = nullptr;
    if (!Value.IsValid() || !Value->TryGetArray(Values) || !Values || Values->Num() != 3) return false;
    double Numbers[3];
    for (int32 I = 0; I < 3; ++I)
    {
        if (!(*Values)[I].IsValid() || !(*Values)[I]->TryGetNumber(Numbers[I]) || !FMath::IsFinite(Numbers[I]) || FMath::Abs(Numbers[I]) > Maximum) return false;
    }
    Out = FVector(Numbers[0], Numbers[1], Numbers[2]);
    return true;
}
bool TripleField(const TSharedPtr<FJsonObject>& Object, const TCHAR* Key, FVector& Out, double Maximum)
{
    return Triple(Object->TryGetField(Key), Out, Maximum);
}
bool Color(const TSharedPtr<FJsonObject>& Object, FLinearColor& Out)
{
    FVector Values;
    if (!TripleField(Object, TEXT("color"), Values, 1.0) || Values.X < 0 || Values.Y < 0 || Values.Z < 0) return false;
    Out = FLinearColor(static_cast<float>(Values.X), static_cast<float>(Values.Y), static_cast<float>(Values.Z));
    return true;
}
bool Version(const FString& Value)
{
    if (Value.IsEmpty() || Value.Len() > 20 || (Value.Len() > 1 && Value[0] == TEXT('0'))) return false;
    for (const TCHAR Character : Value) if (Character < TEXT('0') || Character > TEXT('9')) return false;
    return Value.Len() < 20 || Value <= TEXT("18446744073709551615");
}
bool ReadMesh(const TSharedPtr<FJsonObject>& Object, FMesh& Out, int32& TotalTriangles)
{
    const TArray<TSharedPtr<FJsonValue>>* Triangles = nullptr;
    if (!Text(Object, TEXT("id"), Out.Id) || !Text(Object, TEXT("name"), Out.Name) || !Color(Object, Out.Color) || !Array(Object, TEXT("triangles"), Triangles, 100000)) return false;
    FString Role(TEXT("solid"));
    if (Object->HasField(TEXT("viewRole")) && (!Object->TryGetStringField(TEXT("viewRole"), Role) || (Role != TEXT("solid") && Role != TEXT("enclosureShell")))) return false;
    Out.EnclosureShell = Role == TEXT("enclosureShell");
    TotalTriangles += Triangles->Num();
    if (TotalTriangles > 100000) return false;
    Out.Vertices.Reserve(Triangles->Num() * 3);
    Out.Indices.Reserve(Triangles->Num() * 3);
    Out.Normals.Reserve(Triangles->Num() * 3);
    for (const auto& TriangleValue : *Triangles)
    {
        const TArray<TSharedPtr<FJsonValue>>* Triangle = nullptr;
        if (!TriangleValue->TryGetArray(Triangle) || !Triangle || Triangle->Num() != 3) return false;
        FVector Points[3];
        for (int32 I = 0; I < 3; ++I)
        {
            if (!Triple((*Triangle)[I], Points[I], 110000.0)) return false;
            Points[I] = ToUnreal(Points[I]);
        }
        // UE's procedural meshes use clockwise front faces (see GenerateBoxMesh).
        // Reflecting Y already changes Rust's counter-clockwise winding to that convention.
        const FVector Normal = FVector::CrossProduct(Points[2] - Points[0], Points[1] - Points[0]).GetSafeNormal();
        if (Normal.IsNearlyZero()) return false;
        for (const FVector& Point : Points)
        {
            Out.Indices.Add(Out.Vertices.Num());
            Out.Vertices.Add(Point);
            Out.Normals.Add(Normal);
        }
    }
    return true;
}
bool ReadFixture(const TSharedPtr<FJsonObject>& Object, FFixture& Out)
{
    FVector Origin, Direction;
    double Angle = 0;
    FString Optics, PlacementId;
    if (!Text(Object, TEXT("id"), Out.Id) || !Text(Object, TEXT("name"), Out.Name) ||
        !TripleField(Object, TEXT("originMeters"), Origin, 100000) || !TripleField(Object, TEXT("direction"), Direction, 1.0 + 1e-9) ||
        !Object->TryGetNumberField(TEXT("fullBeamAngleDegrees"), Angle) || !FMath::IsFinite(Angle) || Angle <= 0 || Angle >= 179 ||
        !Text(Object, TEXT("optics"), Optics) || Optics != TEXT("generic-illustrative") ||
        !ObjectField(Object, TEXT("placement"), Out.Placement) || !Text(Out.Placement, TEXT("fixtureId"), PlacementId) || PlacementId != Out.Id ||
        FMath::Abs(Direction.SizeSquared() - 1) > 0.001) return false;
    if (Object->HasField(TEXT("moving")) && !Object->TryGetBoolField(TEXT("moving"), Out.Moving)) return false;
    Out.Origin = ToUnreal(Origin);
    Out.Direction = FVector(Direction.X, -Direction.Y, Direction.Z).GetSafeNormal();
    Out.BeamAngle = static_cast<float>(Angle);
    return true;
}
// Rotation composition can produce a unit component a few ulps above one.
// Tolerance applies only to directions, never colors, levels or coordinates.
bool Basis(const TSharedPtr<FJsonObject>& Object, const TCHAR* Key, FVector& X, FVector& Z)
{
    const TArray<TSharedPtr<FJsonValue>>* Values = nullptr;
    if (!Array(Object, Key, Values, 2) || Values->Num() != 2 || !Triple((*Values)[0], X, 1.0 + 1e-9) || !Triple((*Values)[1], Z, 1.0 + 1e-9) ||
        FMath::Abs(X.SizeSquared()-1)>0.001 || FMath::Abs(Z.SizeSquared()-1)>0.001 || FMath::Abs(FVector::DotProduct(X,Z))>0.001) return false;
    X.Y *= -1; Z.Y *= -1;
    return true;
}
bool ReadPose(const TSharedPtr<FJsonObject>& Data, FJointPose& Pose)
{
    if (!Basis(Data,TEXT("base"),Pose.BaseX,Pose.BaseZ) || !Basis(Data,TEXT("pan"),Pose.PanX,Pose.PanZ) ||
        !Basis(Data,TEXT("head"),Pose.HeadX,Pose.HeadZ) || !TripleField(Data,TEXT("direction"),Pose.Direction,1.0 + 1e-9)) return false;
    Pose.Direction.Y *= -1;
    return Pose.Direction.Equals(-Pose.HeadZ,0.001) && Pose.PanZ.Equals(Pose.BaseZ,0.001) && Pose.HeadX.Equals(Pose.PanX,0.001);
}
FString Decimal(double Value)
{
    FString Result = FString::Printf(TEXT("%.6f"), Value);
    while (Result.EndsWith(TEXT("0"))) Result.LeftChopInline(1);
    if (Result.EndsWith(TEXT("."))) Result.LeftChopInline(1);
    if (Result == TEXT("-0")) Result = TEXT("0");
    return Result;
}
}

FVector ToUnreal(const FVector& Meters) { return FVector(Meters.X, -Meters.Y, Meters.Z) * 100.0; }
FVector ToMeters(const FVector& Unreal) { return FVector(Unreal.X, -Unreal.Y, Unreal.Z) / 100.0; }
bool ReadStamp(const TSharedPtr<FJsonObject>& Object, FStamp& Out)
{
    double Protocol = 0, Generation = 0;
    return Object.IsValid() && Object->TryGetNumberField(TEXT("protocol"), Protocol) && Protocol == 2 &&
        Text(Object, TEXT("bridgeId"), Out.BridgeId) && Text(Object, TEXT("version"), Out.Version) && Version(Out.Version) &&
        Object->TryGetNumberField(TEXT("generation"), Generation) && FMath::IsFinite(Generation) && Generation >= 0 && Generation <= 4294967295.0 &&
        FMath::FloorToDouble(Generation) == Generation && (Out.Generation = static_cast<uint32>(Generation), true);
}
bool ReadScene(const TSharedPtr<FJsonObject>& Object, FScene& Out, FString& Error)
{
    Error = TEXT("场地预演数据格式或容量不正确");
    FScene Next;
    TSharedPtr<FJsonObject> Scene;
    const TArray<TSharedPtr<FJsonValue>> *Meshes = nullptr, *Fixtures = nullptr;
    if (!ReadStamp(Object, Next.Stamp) || !ObjectField(Object, TEXT("scene"), Scene) ||
        !Text(Scene, TEXT("projectId"), Next.ProjectId) || !Text(Scene, TEXT("projectName"), Next.Name) ||
        !Array(Scene, TEXT("meshes"), Meshes, 1024) || !Array(Scene, TEXT("fixtures"), Fixtures, 128)) return false;
    int32 TotalTriangles = 0;
    TSet<FString> Ids;
    for (const auto& Value : *Meshes)
    {
        FMesh Mesh;
        TSharedPtr<FJsonObject> Data;
        if (!ObjectValue(Value, Data) || !ReadMesh(Data, Mesh, TotalTriangles) || Ids.Contains(Mesh.Id)) return false;
        Ids.Add(Mesh.Id);
        Next.Meshes.Add(MoveTemp(Mesh));
    }
    Ids.Reset();
    for (const auto& Value : *Fixtures)
    {
        FFixture Fixture;
        TSharedPtr<FJsonObject> Data;
        if (!ObjectValue(Value, Data) || !ReadFixture(Data, Fixture) || Ids.Contains(Fixture.Id)) return false;
        Ids.Add(Fixture.Id);
        Next.Fixtures.Add(MoveTemp(Fixture));
    }
    Out = MoveTemp(Next);
    Error.Empty();
    return true;
}
bool ReadFrame(const TSharedPtr<FJsonObject>& Object, FFrame& Out, FString& Error)
{
    Error = TEXT("灯光预演数据格式不正确");
    FFrame Next;
    TSharedPtr<FJsonObject> Source;
    const TArray<TSharedPtr<FJsonValue>>* Lights = nullptr;
    if (!ReadStamp(Object, Next.Stamp) || !Text(Object, TEXT("status"), Next.Status) || StatusLabel(Next.Status).IsEmpty() ||
        !ObjectField(Object, TEXT("source"), Source) || !Text(Source, TEXT("kind"), Next.Source) ||
        (Next.Source != TEXT("defaults") && Next.Source != TEXT("scene") && Next.Source != TEXT("playback")) ||
        !Object->TryGetBoolField(TEXT("canEdit"), Next.CanEdit) || !Array(Object, TEXT("lights"), Lights, 128)) return false;
    TSet<FString> Ids;
    for (const auto& Value : *Lights)
    {
        TSharedPtr<FJsonObject> Data;
        FLight Light;
        double Intensity = 0;
        if (!ObjectValue(Value, Data) || !Text(Data, TEXT("fixtureId"), Light.Id) || Ids.Contains(Light.Id) || !Color(Data, Light.Color) ||
            !Data->TryGetNumberField(TEXT("intensity"), Intensity) || !FMath::IsFinite(Intensity) || Intensity < 0 || Intensity > 1) return false;
        if (Data->HasField(TEXT("pose")))
        {
            TSharedPtr<FJsonObject> PoseData;
            FJointPose Pose;
            if (!ObjectField(Data,TEXT("pose"),PoseData) || !ReadPose(PoseData,Pose)) return false;
            Light.Pose = Pose;
        }
        Ids.Add(Light.Id);
        Light.Intensity = static_cast<float>(Intensity);
        Next.Lights.Add(MoveTemp(Light));
    }
    Out = MoveTemp(Next);
    Error.Empty();
    return true;
}
TSharedPtr<FJsonObject> MoveRequest(const FStamp& Stamp, const FFixture& Fixture, const FVector& UnrealLocation)
{
    if (!Fixture.Placement.IsValid() || UnrealLocation.ContainsNaN()) return nullptr;
    const FVector Meters = ToMeters(UnrealLocation);
    if (Meters.GetAbsMax() > 100000) return nullptr;
    auto Position = MakeShared<FJsonObject>();
    Position->SetStringField(TEXT("x"), Decimal(Meters.X));
    Position->SetStringField(TEXT("y"), Decimal(Meters.Y));
    Position->SetStringField(TEXT("z"), Decimal(Meters.Z));
    auto Placement = MakeShared<FJsonObject>();
    Placement->Values = Fixture.Placement->Values;
    Placement->SetObjectField(TEXT("positionMeters"), Position);
    auto Request = MakeShared<FJsonObject>();
    Request->SetStringField(TEXT("bridgeId"), Stamp.BridgeId);
    Request->SetNumberField(TEXT("generation"), Stamp.Generation);
    Request->SetStringField(TEXT("version"), Stamp.Version);
    Request->SetObjectField(TEXT("placement"), Placement);
    return Request;
}
FString StatusLabel(const FString& Status)
{
    if (Status == TEXT("editing")) return TEXT("场景预演");
    if (Status == TEXT("idle")) return TEXT("待执行");
    if (Status == TEXT("running")) return TEXT("正在执行");
    if (Status == TEXT("paused")) return TEXT("已暂停");
    if (Status == TEXT("finished")) return TEXT("本次执行结束");
    if (Status == TEXT("unloaded")) return TEXT("请在桌面载入场景列表");
    if (Status == TEXT("stalePlayback")) return TEXT("工程已修改，请重新载入列表");
    if (Status == TEXT("missingScene")) return TEXT("原场景已删除，请重新选择");
    return FString();
}
}
