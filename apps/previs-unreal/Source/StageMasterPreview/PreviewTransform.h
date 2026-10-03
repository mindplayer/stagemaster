#pragma once
#include "PreviewProtocol.h"
namespace StageMaster {
FVector TransformPoint(const FVector& Point, const FVector& Center, double Yaw, double Scale);
TSharedPtr<FJsonObject> TransformRequest(const FStamp& Stamp, const TArray<FString>& Ids, double Yaw, double Scale);
bool ReadTransform(const TSharedPtr<FJsonObject>& Object, double& Yaw, double& Scale);
}
