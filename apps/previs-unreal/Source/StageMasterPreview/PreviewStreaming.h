#pragma once
#include "CoreMinimal.h"
class APreviewCameraPawn;
class FPreviewStreaming
{
public:
    void Tick(APreviewCameraPawn* Camera);
    void Stop();
private:
    bool Started = false;
    FString LastStatus;
    double NextStatusAt = 0;
};
