#pragma once
#include "CoreMinimal.h"

namespace StageMaster
{
enum class EMarqueeMode : uint8 { Replace, Add, Remove };
// Only screen-space inclusion and ordered selection policy; no document mutation.
bool InsideMarquee(const FVector2D& Point, const FVector2D& Start, const FVector2D& End, const FVector2D& Viewport);
bool MarqueeSelection(const TArray<FString>& Before, const TArray<FString>& Hits, EMarqueeMode Mode, TArray<FString>& Out);
struct FMarqueeGesture
{
    bool Active = false;
    bool Moved = false;
    bool ClickAdditive = false;
    uint64 Serial = 0;
    FVector2D Start = FVector2D::ZeroVector;
    FVector2D End = FVector2D::ZeroVector;
    EMarqueeMode Mode = EMarqueeMode::Replace;
    void Begin(const FVector2D& Point, uint64 Source, bool Additive, bool Remove, EMarqueeMode DefaultMode = EMarqueeMode::Replace);
    void Update(const FVector2D& Point);
    void Cancel() { Active = Moved = false; }
};
}
