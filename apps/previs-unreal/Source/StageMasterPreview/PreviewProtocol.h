#pragma once

#include "CoreMinimal.h"
#include "Dom/JsonObject.h"

namespace StageMaster
{
// Transport DTOs only. Rust owns installations, scene semantics and playback time.
struct FStamp
{
    FString BridgeId;
    FString Version;
    uint32 Generation = 0;
};
struct FMesh
{
    FString Id;
    FString Name;
    FLinearColor Color;
    TArray<FVector> Vertices;
    TArray<int32> Indices;
    TArray<FVector> Normals;
};
struct FFixture
{
    FString Id;
    FString Name;
    FVector Origin;
    FVector Direction;
    float BeamAngle = 0;
    TSharedPtr<FJsonObject> Placement;
};
struct FScene
{
    FStamp Stamp;
    FString ProjectId;
    FString Name;
    TArray<FMesh> Meshes;
    TArray<FFixture> Fixtures;
};
struct FLight
{
    FString Id;
    float Intensity = 0;
    FLinearColor Color;
};
struct FFrame
{
    FStamp Stamp;
    FString Status;
    FString Source;
    bool CanEdit = false;
    TArray<FLight> Lights;
};

// Conversion is confined to this boundary: right-handed metres -> UE centimetres.
FVector ToUnreal(const FVector& Meters);
FVector ToMeters(const FVector& Unreal);
bool ReadStamp(const TSharedPtr<FJsonObject>& Object, FStamp& Out);
bool ReadScene(const TSharedPtr<FJsonObject>& Object, FScene& Out, FString& Error);
bool ReadFrame(const TSharedPtr<FJsonObject>& Object, FFrame& Out, FString& Error);
TSharedPtr<FJsonObject> MoveRequest(const FStamp& Stamp, const FFixture& Fixture, const FVector& UnrealLocation);
FString StatusLabel(const FString& Status);
}
