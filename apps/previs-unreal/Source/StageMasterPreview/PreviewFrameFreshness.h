#pragma once
#include "CoreMinimal.h"

namespace StageMaster
{
// Local read contention may skip a frame, but never extends the last valid frame's lease.
inline bool HoldBusyFrame(int32 StatusCode, bool FrameRequest, bool WasValid, double LastFrameAt, double Now)
{
    const double Age = Now - LastFrameAt;
    return StatusCode == 503 && FrameRequest && WasValid &&
        FMath::IsFinite(Age) && Age >= 0 && Age < 2.0;
}
}
