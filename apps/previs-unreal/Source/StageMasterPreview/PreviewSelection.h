#pragma once
#include "PreviewProtocol.h"

namespace StageMaster
{
// Bounded interaction helpers shared by native input and the host transport.
bool ReadFixtureSelection(const TSharedPtr<FJsonObject>& Object, TArray<FString>& Out);
TArray<FString> SelectFixture(const TArray<FString>& Before, const FString& Hit, bool Additive, bool Moving);
bool DragPlanePoint(const FVector& RayOrigin, const FVector& RayDirection, const FVector& Anchor,
    const FVector& Normal, FVector& Out);
TSharedPtr<FJsonObject> TranslationRequest(const FStamp& Stamp, const TArray<FString>& Ids, const FVector& Delta);
}
