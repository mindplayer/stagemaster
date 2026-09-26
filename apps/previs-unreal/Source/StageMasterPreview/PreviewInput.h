#pragma once
#include "CoreMinimal.h"
#include "Serialization/MemoryReader.h"

namespace StageMaster
{
// Pixel Streaming's normalized MouseUp payload. Leave the reader untouched so
// the official handler still performs button release and capture cleanup.
inline bool ReadMouseRelease(FMemoryReader& Reader, int32 Width, int32 Height, uint8& Button, FVector2D& Position, bool& InRange)
{
    const int64 Offset = Reader.Tell();
    if (Reader.TotalSize() - Offset != 5 || Width <= 0 || Height <= 0) return false;
    uint8 ParsedButton = 0;
    uint16 X = 0, Y = 0;
    Reader << ParsedButton << X << Y;
    Reader.Seek(Offset);
    if (ParsedButton > 4) return false;
    Button = ParsedButton;
    // InputCoordTranslator uses this pair for letterboxing/outside coordinates.
    InRange = X != 65535 || Y != 65535;
    Position = FVector2D(X * Width / 65535.0, Y * Height / 65535.0);
    return true;
}
}
