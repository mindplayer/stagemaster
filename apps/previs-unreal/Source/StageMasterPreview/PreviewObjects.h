#pragma once
#include "PreviewProtocol.h"

namespace StageMaster
{
// These keys are renderer-local only. Wire messages always contain kind + id.
FString FixtureKey(const FString& Id);
FString ConstructionKey(const FString& Id);
bool SplitObjectKey(const FString& Key, FString& Kind, FString& Id);
TArray<FString> FixtureIds(const TArray<FString>& Keys);
bool ReadObjectSelection(const TSharedPtr<FJsonObject>& Object, TArray<FString>& Out);
TArray<TSharedPtr<FJsonValue>> ObjectValues(const TArray<FString>& Keys);
bool ReadMeshOwner(const TSharedPtr<FJsonObject>& Object, FMesh& Out);
bool ValidObjectOwners(const FScene& Scene);
TSharedPtr<FJsonObject> ObjectTranslationRequest(const FStamp& Stamp, const TArray<FString>& Keys, const FVector& Delta);
}
