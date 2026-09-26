#include "PreviewStreaming.h"
#include "IPixelStreaming2Module.h"
#include "IPixelStreaming2Streamer.h"
#include "IPixelStreaming2InputHandler.h"
#include "Misc/Char.h"
#include "PreviewCameraPawn.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonWriter.h"

void FPreviewStreaming::Tick(APreviewCameraPawn* Camera)
{
    auto& Module = IPixelStreaming2Module::Get();
    if (!Module.IsReady()) return;
    auto Streamer = Module.FindStreamer(Module.GetDefaultStreamerID());
    if (!Streamer.IsValid()) return;
    if (Started)
    {
        const double Now = FPlatformTime::Seconds();
        if (Now < NextStatusAt) return;
        NextStatusAt = Now + 0.2;
        const auto State = MakeShared<FJsonObject>();
        State->SetStringField(TEXT("status"), Camera->StatusText().ToString());
        State->SetStringField(TEXT("selection"), Camera->SelectionText().ToString());
        State->SetStringField(TEXT("workLight"), Camera->WorkLightText().ToString());
        FString Json;
        FJsonSerializer::Serialize(State, TJsonWriterFactory<>::Create(&Json));
        // Send periodically as viewers may reconnect while the scene stays unchanged.
        Streamer->SendAllPlayersMessage(TEXT("Response"), Json);
        return;
    }
    const FString Url = FPlatformMisc::GetEnvironmentVariable(TEXT("STAGEMASTER_STREAM_URL"));
    const FString Prefix(TEXT("ws://127.0.0.1:"));
    FString Port, Token;
    if (!Url.StartsWith(Prefix) || !Url.Mid(Prefix.Len()).Split(TEXT("/"), &Port, &Token) ||
        Port.IsEmpty() || Port.Len() > 5 || Token.Len() != 64) return;
    for (const TCHAR C : Port) if (!FChar::IsDigit(C)) return;
    for (const TCHAR C : Token) if (!FChar::IsHexDigit(C)) return;
    if (FCString::Atoi(*Port) < 1 || FCString::Atoi(*Port) > 65535) return;
    if (auto Input = Streamer->GetInputHandler().Pin())
    {
        // No renderer console or arbitrary command entry point from the WebView.
        Input->RegisterMessageHandler(TEXT("Command"), [](FString, FMemoryReader) {});
        const TWeakObjectPtr<APreviewCameraPawn> WeakCamera(Camera);
        Input->RegisterMessageHandler(TEXT("UIInteraction"), [WeakCamera](FString, FMemoryReader Ar) {
            if (!WeakCamera.IsValid() || Ar.TotalSize() - Ar.Tell() < 2) return;
            uint16 Length = 0;
            Ar << Length;
            if (Length == 0 || Length > 256 || Ar.TotalSize() - Ar.Tell() != Length * sizeof(TCHAR)) return;
            FString Json;
            Json.GetCharArray().SetNumZeroed(Length + 1);
            Ar.Serialize(Json.GetCharArray().GetData(), Length * sizeof(TCHAR));
            TSharedPtr<FJsonObject> Object;
            if (!FJsonSerializer::Deserialize(TJsonReaderFactory<>::Create(Json), Object) || !Object.IsValid() || Object->Values.Num() != 1) return;
            FString Action;
            if (Object->TryGetStringField(TEXT("action"), Action)) WeakCamera->ViewAction(Action);
        });
    }
    Streamer->SetConnectionURL(Url);
    Streamer->StartStreaming();
    Started = true;
}
void FPreviewStreaming::Stop()
{
    if (Started && IPixelStreaming2Module::IsAvailable()) IPixelStreaming2Module::Get().StopStreaming();
    Started = false;
}
