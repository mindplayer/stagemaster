#pragma once
#include "CoreMinimal.h"
class APreviewCameraPawn;
class FJsonObject;
class FPreviewStreaming
{
public:
    void Tick(APreviewCameraPawn* Camera);
    void Stop();
    bool Send(const TSharedPtr<FJsonObject>& Message);
private:
    bool Started = false;
    double NextStatusAt = 0;
};
