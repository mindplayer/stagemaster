#include "PreviewMarquee.h"

namespace StageMaster
{
bool InsideMarquee(const FVector2D& Point, const FVector2D& Start, const FVector2D& End, const FVector2D& Viewport)
{
    if (Point.ContainsNaN() || Start.ContainsNaN() || End.ContainsNaN() || Viewport.ContainsNaN() ||
        Viewport.X <= 0 || Viewport.Y <= 0) return false;
    return Point.X >= 0 && Point.Y >= 0 && Point.X <= Viewport.X && Point.Y <= Viewport.Y &&
        Point.X >= FMath::Min(Start.X, End.X) && Point.X <= FMath::Max(Start.X, End.X) &&
        Point.Y >= FMath::Min(Start.Y, End.Y) && Point.Y <= FMath::Max(Start.Y, End.Y);
}
bool MarqueeSelection(const TArray<FString>& Before, const TArray<FString>& Hits, EMarqueeMode Mode, TArray<FString>& Out)
{
    if (Before.Num() > 1024 || Hits.Num() > 1024) return false;
    TSet<FString> Unique;
    for (const auto* Group : {&Before, &Hits})
    {
        Unique.Empty();
        for (const auto& Id : *Group)
        {
            if (Id.IsEmpty() || Id.Len() > 258 || Unique.Contains(Id)) return false;
            Unique.Add(Id);
        }
    }
    auto Result = Mode == EMarqueeMode::Replace ? Hits : Before;
    if (Mode == EMarqueeMode::Add) for (const auto& Id : Hits) Result.AddUnique(Id);
    if (Mode == EMarqueeMode::Remove) Result.RemoveAll([&Unique](const FString& Id) { return Unique.Contains(Id); });
    if (Result.Num() > 1024) return false;
    Out = MoveTemp(Result);
    return true;
}
void FMarqueeGesture::Begin(const FVector2D& Point, uint64 Source, bool Additive, bool Remove, EMarqueeMode DefaultMode)
{
    Active = true;
    Moved = false;
    Serial = Source;
    Start = End = Point;
    ClickAdditive = Additive || Remove;
    Mode = Remove ? EMarqueeMode::Remove : Additive ? EMarqueeMode::Add : DefaultMode;
}
void FMarqueeGesture::Update(const FVector2D& Point)
{
    if (!Active) return;
    if (Point.ContainsNaN()) { Cancel(); return; }
    End = Point;
    Moved |= (End - Start).SizeSquared() >= 9;
}
}
