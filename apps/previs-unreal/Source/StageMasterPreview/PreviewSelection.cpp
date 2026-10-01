#include "PreviewSelection.h"

namespace StageMaster
{
bool ReadFixtureSelection(const TSharedPtr<FJsonObject>& Object, TArray<FString>& Out)
{
    const TArray<TSharedPtr<FJsonValue>>* Values = nullptr;
    if (!Object.IsValid() || !Object->TryGetArrayField(TEXT("fixtureIds"), Values) || Values->Num() > 1024) return false;
    TArray<FString> Result;
    TSet<FString> Unique;
    for (const auto& Value : *Values)
    {
        FString Id;
        if (!Value.IsValid() || !Value->TryGetString(Id) || Id.IsEmpty() || Id.Len() > 256 || Unique.Contains(Id)) return false;
        Unique.Add(Id);
        Result.Add(Id);
    }
    Out = MoveTemp(Result);
    return true;
}
TArray<FString> SelectFixture(const TArray<FString>& Before, const FString& Hit, bool Additive, bool Moving)
{
    if (Additive)
    {
        auto After = Before;
        if (Hit.IsEmpty()) return After;
        if (After.Contains(Hit)) After.Remove(Hit);
        else if (After.Num() < 1024) After.Add(Hit);
        return After;
    }
    if (Moving && Before.Contains(Hit)) return Before;
    return Hit.IsEmpty() ? TArray<FString>() : TArray<FString>{Hit};
}
bool DragPlanePoint(const FVector& RayOrigin, const FVector& RayDirection, const FVector& Anchor,
    const FVector& Normal, FVector& Out)
{
    if (RayOrigin.ContainsNaN() || RayDirection.ContainsNaN() || Anchor.ContainsNaN() || Normal.ContainsNaN()) return false;
    const double Denominator = FVector::DotProduct(RayDirection, Normal);
    if (FMath::Abs(Denominator) < 0.0001) return false;
    const double T = FVector::DotProduct(Anchor - RayOrigin, Normal) / Denominator;
    if (!FMath::IsFinite(T) || T < 0 || T > 20000000) return false;
    Out = RayOrigin + RayDirection * T;
    return !Out.ContainsNaN();
}
TSharedPtr<FJsonObject> TranslationRequest(const FStamp& Stamp, const TArray<FString>& Ids, const FVector& Delta)
{
    if (Ids.IsEmpty() || Ids.Num() > 256 || Delta.ContainsNaN() || Delta.GetAbsMax() > 20000000) return nullptr;
    TSet<FString> Unique;
    TArray<TSharedPtr<FJsonValue>> Values;
    for (const auto& Id : Ids)
    {
        if (Id.IsEmpty() || Id.Len() > 256 || Unique.Contains(Id)) return nullptr;
        Unique.Add(Id);
        Values.Add(MakeShared<FJsonValueString>(Id));
    }
    const auto Request = MakeShared<FJsonObject>();
    Request->SetNumberField(TEXT("generation"), Stamp.Generation);
    Request->SetStringField(TEXT("version"), Stamp.Version);
    Request->SetArrayField(TEXT("fixtureIds"), Values);
    const FVector Meters = ToMeters(Delta);
    const auto Vector = MakeShared<FJsonObject>();
    for (const auto& Axis : {TPair<FString, double>(TEXT("x"), Meters.X), {TEXT("y"), Meters.Y}, {TEXT("z"), Meters.Z}})
    {
        FString Value = FString::Printf(TEXT("%.6f"), Axis.Value);
        while (Value.EndsWith(TEXT("0"))) Value.LeftChopInline(1);
        if (Value.EndsWith(TEXT("."))) Value.LeftChopInline(1);
        if (Value == TEXT("-0")) Value = TEXT("0");
        Vector->SetStringField(Axis.Key, Value);
    }
    Request->SetObjectField(TEXT("deltaMeters"), Vector);
    return Request;
}
}
