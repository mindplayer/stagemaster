#include "PreviewBridge.h"
#include "HttpModule.h"
#include "Interfaces/IHttpResponse.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonWriter.h"

namespace StageMaster
{
bool FPreviewBridge::Start()
{
    Stop();
    BaseUrl = FPlatformMisc::GetEnvironmentVariable(TEXT("STAGEMASTER_PREVIS_URL"));
    Authorization = FPlatformMisc::GetEnvironmentVariable(TEXT("STAGEMASTER_PREVIS_AUTHORIZATION"));
    ExpectedBridgeId = FPlatformMisc::GetEnvironmentVariable(TEXT("STAGEMASTER_PREVIS_SESSION"));
    // The renderer is not a general-purpose HTTP client. Never forward the credential elsewhere.
    const FString Prefix(TEXT("http://127.0.0.1:"));
    const FString Port = BaseUrl.Mid(Prefix.Len());
    bool DigitsOnly = !Port.IsEmpty() && Port.Len() <= 5;
    for (const TCHAR C : Port) DigitsOnly &= C >= TEXT('0') && C <= TEXT('9');
    if (!BaseUrl.StartsWith(Prefix) || !DigitsOnly || FCString::Atoi(*Port) < 1 || FCString::Atoi(*Port) > 65535 ||
        !Authorization.StartsWith(TEXT("Bearer ")) || Authorization.Len() != 71 || ExpectedBridgeId.Len() != 36)
    {
        Fail(TEXT("请从舞台大师打开三维预演"));
        return false;
    }
    Running = true;
    NeedScene = true;
    NextRequestAt = 0;
    LastFrameAt = 0;
    Version.Empty();
    Message = TEXT("正在读取场地");
    return true;
}
void FPreviewBridge::Stop()
{
    Running = false;
    WasValid = false;
    if (Pending.IsValid())
    {
        Pending->OnProcessRequestComplete().Unbind();
        Pending->CancelRequest();
        Pending.Reset();
    }
    Authorization.Empty();
    FixtureIds.Reset();
}
void FPreviewBridge::Tick(double Now)
{
    if (!Running) return;
    if (WasValid && Now - LastFrameAt > 2)
    {
        WasValid = false;
        Fail(TEXT("预演连接中断，正在重新连接"));
    }
    if (Pending.IsValid() || Now < NextRequestAt) return;
    Request(NeedScene ? ERequest::Scene : ERequest::Frame);
}
bool FPreviewBridge::SubmitPlacement(const TSharedPtr<FJsonObject>& Body)
{
    if (!Running || NeedScene || !WasValid || !Body.IsValid()) return false;
    if (Pending.IsValid())
    {
        if (PendingKind != ERequest::Frame) return false;
        Pending->OnProcessRequestComplete().Unbind();
        Pending->CancelRequest();
        Pending.Reset();
    }
    Request(ERequest::Placement, Body);
    return Pending.IsValid();
}
void FPreviewBridge::Request(ERequest Kind, const TSharedPtr<FJsonObject>& Body)
{
    check(IsInGameThread());
    const TCHAR* Path = Kind == ERequest::Scene ? TEXT("/v1/scene") : Kind == ERequest::Frame ? TEXT("/v1/frame") : TEXT("/v1/placement");
    auto Http = FHttpModule::Get().CreateRequest();
    Http->SetURL(BaseUrl + Path);
    Http->SetVerb(Kind == ERequest::Placement ? TEXT("POST") : TEXT("GET"));
    Http->SetHeader(TEXT("Authorization"), Authorization);
    Http->SetHeader(TEXT("Accept"), TEXT("application/json"));
    Http->SetTimeout(3.0f);
    Http->SetDelegateThreadPolicy(EHttpRequestDelegateThreadPolicy::CompleteOnGameThread);
    if (Body.IsValid())
    {
        FString Json;
        FJsonSerializer::Serialize(Body.ToSharedRef(), TJsonWriterFactory<>::Create(&Json));
        Http->SetHeader(TEXT("Content-Type"), TEXT("application/json"));
        Http->SetContentAsString(Json);
    }
    TWeakPtr<FPreviewBridge> Weak = AsShared();
    Http->OnProcessRequestComplete().BindLambda([Weak, Kind](FHttpRequestPtr Completed, FHttpResponsePtr Response, bool Success)
    {
        if (auto Self = Weak.Pin()) Self->Complete(Kind, Completed, Response, Success);
    });
    Pending = Http;
    PendingKind = Kind;
    if (!Http->ProcessRequest())
    {
        Pending.Reset();
        Fail(TEXT("无法连接舞台大师"));
    }
}
void FPreviewBridge::Fail(const FString& Reason, double RetryAfter)
{
    Message = Reason;
    WasValid = false;
    NextRequestAt = FPlatformTime::Seconds() + RetryAfter;
    if (OnInvalidated) OnInvalidated(Reason);
}
void FPreviewBridge::Complete(ERequest Kind, FHttpRequestPtr Completed, FHttpResponsePtr Response, bool Success)
{
    check(IsInGameThread());
    if (!Running || Completed != Pending) return;
    Pending.Reset();
    NextRequestAt = FPlatformTime::Seconds() + 1.0 / 30.0;
    if (!Success || !Response.IsValid())
    {
        Fail(Kind == ERequest::Placement ? TEXT("灯位提交未确认，正在读取实际位置") : TEXT("预演连接中断，正在重新连接"));
        NeedScene = true;
        return;
    }
    const int32 Code = Response->GetResponseCode();
    if (Code == 403)
    {
        Running = false;
        Fail(TEXT("连接已关闭，请从舞台大师重新打开预演"));
        return;
    }
    // The server emits a bounded payload; reject unexpected peers before parsing JSON.
    if (Response->GetContent().Num() > (Kind == ERequest::Scene ? 32 * 1024 * 1024 : 256 * 1024))
    {
        Fail(TEXT("预演数据超出允许大小"), 2);
        return;
    }
    TSharedPtr<FJsonObject> Object;
    if (!FJsonSerializer::Deserialize(TJsonReaderFactory<>::Create(Response->GetContentAsString()), Object) || !Object.IsValid())
    {
        Fail(TEXT("无法读取预演响应"));
        return;
    }
    if (Code != 200)
    {
        FString Reason;
        if (!Object->TryGetStringField(TEXT("message"), Reason) || Reason.IsEmpty() || Reason.Len() > 4096) Reason = TEXT("预演暂不可用，请重试");
        if (Code == 409 || Kind == ERequest::Placement) NeedScene = true;
        Fail(Reason);
        return;
    }
    if (Kind == ERequest::Scene) AcceptScene(Object);
    else if (Kind == ERequest::Frame) AcceptFrame(Object);
    else
    {
        FStamp Stamp;
        if (!ReadStamp(Object, Stamp) || Stamp.BridgeId != ExpectedBridgeId)
        {
            Fail(TEXT("灯位提交结果无效，正在读取实际位置"));
        }
        else Message = TEXT("灯位已更新");
        NeedScene = true;
    }
}
bool FPreviewBridge::AcceptScene(const TSharedPtr<FJsonObject>& Object)
{
    FScene Scene;
    FString Error;
    if (!ReadScene(Object, Scene, Error) || Scene.Stamp.BridgeId != ExpectedBridgeId)
    {
        Fail(Error.IsEmpty() ? TEXT("预演会话不匹配") : Error, 2);
        return false;
    }
    Version = Scene.Stamp.Version;
    Generation = Scene.Stamp.Generation;
    FixtureIds.Reset();
    for (const auto& Fixture : Scene.Fixtures) FixtureIds.Add(Fixture.Id);
    NeedScene = false;
    WasValid = false;
    if (OnScene) OnScene(MoveTemp(Scene));
    return true;
}
bool FPreviewBridge::AcceptFrame(const TSharedPtr<FJsonObject>& Object)
{
    FFrame Frame;
    FString Error;
    if (!ReadFrame(Object, Frame, Error) || Frame.Stamp.BridgeId != ExpectedBridgeId || Frame.Stamp.Version != Version || Frame.Stamp.Generation != Generation)
    {
        NeedScene = true;
        Fail(Error.IsEmpty() ? TEXT("场地已变化，正在重新读取") : Error);
        return false;
    }
    const bool Ready = Frame.Status != TEXT("unloaded") && Frame.Status != TEXT("missingScene") && Frame.Status != TEXT("stalePlayback");
    if (Ready && Frame.Lights.Num() != FixtureIds.Num())
    {
        NeedScene = true;
        Fail(TEXT("灯光与场地不一致，正在重新读取"));
        return false;
    }
    for (const auto& Light : Frame.Lights)
    {
        if (!FixtureIds.Contains(Light.Id)) { NeedScene = true; Fail(TEXT("灯光与场地不一致")); return false; }
    }
    LastFrameAt = FPlatformTime::Seconds();
    WasValid = true;
    Message = StatusLabel(Frame.Status);
    if (OnFrame) OnFrame(MoveTemp(Frame));
    return true;
}
}
