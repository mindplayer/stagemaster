#include "PreviewTransform.h"
#include "PreviewSelection.h"
namespace StageMaster {
FVector TransformPoint(const FVector& Point, const FVector& Center, double Yaw, double Scale)
{
    // Project +Y is Unreal -Y: a positive project yaw is negative here.
    return Center + ((Point - Center) * Scale).RotateAngleAxis(-Yaw, FVector::UpVector);
}
static FString Decimal(double Value)
{
    FString Text = FString::Printf(TEXT("%.6f"), Value);
    while (Text.EndsWith(TEXT("0"))) Text.LeftChopInline(1);
    if (Text.EndsWith(TEXT("."))) Text.LeftChopInline(1);
    return Text == TEXT("-0") ? TEXT("0") : Text;
}
TSharedPtr<FJsonObject> TransformRequest(const FStamp& Stamp, const TArray<FString>& Ids, double Yaw, double Scale)
{
    if (!FMath::IsFinite(Yaw) || FMath::Abs(Yaw) > 360 || !FMath::IsFinite(Scale) || Scale < 0.01 || Scale > 100) return nullptr;
    auto Request = TranslationRequest(Stamp, Ids, FVector::ZeroVector);
    if (!Request.IsValid()) return nullptr;
    Request->RemoveField(TEXT("deltaMeters"));
    Request->SetStringField(TEXT("yawDegrees"), Decimal(Yaw));
    Request->SetStringField(TEXT("spacingScale"), Decimal(Scale));
    return Request;
}
static bool ReadScalar(const TSharedPtr<FJsonObject>& Object, const TCHAR* Key, double Min, double Max, double& Out)
{
    FString Text;
    if (!Object.IsValid() || !Object->TryGetStringField(Key, Text) || Text.IsEmpty() || Text.Len() > 32) return false;
    int32 Digits = 0, Fraction = -1;
    for (int32 I = Text[0] == TCHAR('-') ? 1 : 0; I < Text.Len(); ++I)
    {
        const TCHAR C = Text[I];
        if (C == TCHAR('.') && Fraction == -1 && Digits > 0) { Fraction = 0; continue; }
        if (C < TCHAR('0') || C > TCHAR('9')) return false;
        ++Digits;
        if (Fraction >= 0 && ++Fraction > 6) return false;
    }
    if (Digits == 0 || Fraction == 0) return false;
    const double Value = FCString::Atod(*Text);
    if (!FMath::IsFinite(Value) || Value < Min || Value > Max) return false;
    Out = Value;
    return true;
}
bool ReadTransform(const TSharedPtr<FJsonObject>& Object, double& Yaw, double& Scale)
{
    return ReadScalar(Object, TEXT("yawDegrees"), -360, 360, Yaw)
        && ReadScalar(Object, TEXT("spacingScale"), 0.01, 100, Scale);
}
}
