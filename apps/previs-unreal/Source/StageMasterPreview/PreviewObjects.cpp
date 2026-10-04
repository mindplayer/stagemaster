#include "PreviewObjects.h"
#include "PreviewSelection.h"

namespace StageMaster
{
FString FixtureKey(const FString& Id) { return TEXT("p:") + Id; }
FString ConstructionKey(const FString& Id) { return TEXT("c:") + Id; }
bool SplitObjectKey(const FString& Key, FString& Kind, FString& Id)
{
    if (Key.Len() < 3 || Key.Len() > 258 || (!Key.StartsWith(TEXT("p:")) && !Key.StartsWith(TEXT("c:")))) return false;
    Kind = Key.StartsWith(TEXT("p:")) ? TEXT("placement") : TEXT("construction");
    Id = Key.Mid(2);
    return true;
}
TArray<FString> FixtureIds(const TArray<FString>& Keys)
{
    TArray<FString> Ids;
    for (const auto& Key : Keys) if (Key.StartsWith(TEXT("p:"))) Ids.Add(Key.Mid(2));
    return Ids;
}
bool ReadObjectSelection(const TSharedPtr<FJsonObject>& Object, TArray<FString>& Out)
{
    const TArray<TSharedPtr<FJsonValue>>* Values = nullptr;
    if (!Object.IsValid() || !Object->TryGetArrayField(TEXT("targets"), Values) || Values->Num() > 1024) return false;
    TArray<FString> Result;
    TSet<FString> Seen;
    for (const auto& Value : *Values)
    {
        const TSharedPtr<FJsonObject>* Entry = nullptr;
        FString Kind, Id;
        if (!Value.IsValid() || !Value->TryGetObject(Entry) || !Entry || !Entry->IsValid() || (*Entry)->Values.Num() != 2 ||
            !(*Entry)->TryGetStringField(TEXT("kind"), Kind) || !(*Entry)->TryGetStringField(TEXT("id"), Id) ||
            Id.IsEmpty() || Id.Len() > 256 || (Kind != TEXT("placement") && Kind != TEXT("construction"))) return false;
        const FString Key = Kind == TEXT("placement") ? FixtureKey(Id) : ConstructionKey(Id);
        if (Seen.Contains(Key)) return false;
        Seen.Add(Key); Result.Add(Key);
    }
    Out = MoveTemp(Result);
    return true;
}
TArray<TSharedPtr<FJsonValue>> ObjectValues(const TArray<FString>& Keys)
{
    TArray<TSharedPtr<FJsonValue>> Values;
    for (const auto& Key : Keys)
    {
        FString Kind, Id;
        if (!SplitObjectKey(Key, Kind, Id)) return {};
        auto Value = MakeShared<FJsonObject>();
        Value->SetStringField(TEXT("kind"), Kind); Value->SetStringField(TEXT("id"), Id);
        Values.Add(MakeShared<FJsonValueObject>(Value));
    }
    return Values;
}
bool ReadMeshOwner(const TSharedPtr<FJsonObject>& Object, FMesh& Out)
{
    if (!Object->HasField(TEXT("constructionId")))
        return !Object->HasField(TEXT("movable")) && !Object->HasField(TEXT("attachedFixtureIds"));
    const TArray<TSharedPtr<FJsonValue>>* Attached = nullptr;
    if (!Object->TryGetStringField(TEXT("constructionId"), Out.ConstructionId) || Out.ConstructionId.IsEmpty() || Out.ConstructionId.Len() > 256 ||
        !Object->TryGetBoolField(TEXT("movable"), Out.Movable) ||
        !Object->TryGetArrayField(TEXT("attachedFixtureIds"), Attached) || Attached->Num() > 1024) return false;
    for (const auto& Value : *Attached)
    {
        FString Id;
        if (!Value.IsValid() || !Value->TryGetString(Id) || Id.IsEmpty() || Id.Len() > 256 || Out.AttachedFixtureIds.Contains(Id)) return false;
        Out.AttachedFixtureIds.Add(Id);
    }
    return Out.Movable || Out.AttachedFixtureIds.IsEmpty();
}
bool ValidObjectOwners(const FScene& Scene)
{
    TMap<FString, const FMesh*> Owners;
    TMap<FString, FString> Attachments;
    for (const auto& Mesh : Scene.Meshes)
    {
        if (Mesh.ConstructionId.IsEmpty()) continue;
        if (const auto Earlier = Owners.Find(Mesh.ConstructionId))
        {
            if ((*Earlier)->Movable != Mesh.Movable || (*Earlier)->Name != Mesh.Name || (*Earlier)->AttachedFixtureIds != Mesh.AttachedFixtureIds) return false;
        }
        Owners.Add(Mesh.ConstructionId, &Mesh);
        for (const auto& Id : Mesh.AttachedFixtureIds)
        {
            if (!Scene.Fixtures.ContainsByPredicate([&](const auto& F) { return F.Id == Id; })) return false;
            if (const auto Owner = Attachments.Find(Id); Owner && *Owner != Mesh.ConstructionId) return false;
            Attachments.Add(Id, Mesh.ConstructionId);
        }
    }
    return true;
}
TSharedPtr<FJsonObject> ObjectTranslationRequest(const FStamp& Stamp, const TArray<FString>& Keys, const FVector& Delta)
{
    if (Keys.IsEmpty() || Keys.Num() > 256) return nullptr;
    const auto Values = ObjectValues(Keys);
    if (Values.Num() != Keys.Num()) return nullptr;
    auto Check = MakeShared<FJsonObject>(); Check->SetArrayField(TEXT("targets"), Values);
    TArray<FString> Valid;
    if (!ReadObjectSelection(Check, Valid)) return nullptr;
    // Reuse the existing bounded decimal/revision serializer, replacing its identity field.
    auto Request = TranslationRequest(Stamp, {TEXT("object")}, Delta);
    if (!Request.IsValid()) return nullptr;
    Request->RemoveField(TEXT("fixtureIds")); Request->SetArrayField(TEXT("targets"), Values);
    return Request;
}
}
