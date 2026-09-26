#include "PreviewStreaming.h"
#include "IPixelStreaming2Module.h"
#include "IPixelStreaming2Streamer.h"
#include "IPixelStreaming2InputHandler.h"
#include "Misc/Char.h"
#include "PreviewCameraPawn.h"
#include "PreviewInput.h"
#include "GameFramework/PlayerController.h"
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
        State->SetStringField(TEXT("kind"), TEXT("state"));
        State->SetStringField(TEXT("status"), Camera->StatusText().ToString());
        State->SetStringField(TEXT("selection"), Camera->SelectionText().ToString());
        State->SetStringField(TEXT("workLight"), Camera->WorkLightText().ToString());
        State->SetBoolField(TEXT("move"), Camera->IsMoveMode());
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
        const auto MouseLeave = Input->FindMessageHandler(TEXT("MouseLeave"));
        Input->RegisterMessageHandler(TEXT("MouseLeave"), [WeakCamera, MouseLeave](FString Source, FMemoryReader Ar) {
            if (WeakCamera.IsValid()) WeakCamera->CancelDrag();
            if (MouseLeave) MouseLeave(Source, Ar);
        });
        const auto MouseUp = Input->FindMessageHandler(TEXT("MouseUp"));
        // UE 5.8's default MouseUp discards its final coordinates. Fast/coalesced
        // gestures may deliver MouseMove on the following tick; apply the endpoint first.
        Input->RegisterMessageHandler(TEXT("MouseUp"), [WeakCamera, MouseUp](FString Source, FMemoryReader Ar) {
            if (!WeakCamera.IsValid()) return;
            if (auto Player = Cast<APlayerController>(WeakCamera->GetController()))
            {
                int32 Width = 0, Height = 0;
                Player->GetViewportSize(Width, Height);
                uint8 Button = 0;
                FVector2D Position;
                bool InRange = false;
                if (!StageMaster::ReadMouseRelease(Ar, Width, Height, Button, Position, InRange)) return;
                if (!InRange) WeakCamera->CancelDrag();
                else if (Button == 0) WeakCamera->DragTo(Position);
                if (MouseUp) MouseUp(Source, Ar);
            }
        });
        Input->RegisterMessageHandler(TEXT("UIInteraction"), [WeakCamera](FString, FMemoryReader Ar) {
            if (!WeakCamera.IsValid() || Ar.TotalSize() - Ar.Tell() < 2) return;
            uint16 Length = 0;
            Ar << Length;
            if (Length == 0 || Length > 256 || Ar.TotalSize() - Ar.Tell() != Length * sizeof(TCHAR)) return;
            FString Json;
            Json.GetCharArray().SetNumZeroed(Length + 1);
            Ar.Serialize(Json.GetCharArray().GetData(), Length * sizeof(TCHAR));
            TSharedPtr<FJsonObject> Object;
            if (!FJsonSerializer::Deserialize(TJsonReaderFactory<>::Create(Json), Object) || !Object.IsValid()) return;
            FString Action;
            if (!Object->TryGetStringField(TEXT("action"), Action)) return;
            if (Object->Values.Num() == 1) WeakCamera->ViewAction(Action);
            else if (Action == TEXT("select") && Object->Values.Num() == 2)
            {
                FString Id;
                if (Object->TryGetStringField(TEXT("fixtureId"), Id) && Id.Len() <= 256) WeakCamera->SelectFromHost(Id);
            }
            else if (Action == TEXT("placementResult") && Object->Values.Num() == 3)
            {
                FString Id;
                bool Accepted = false;
                if (Object->TryGetStringField(TEXT("requestId"), Id) && Id.Len() == 32 && Object->TryGetBoolField(TEXT("accepted"), Accepted))
                    WeakCamera->PlacementResult(Id, Accepted);
            }
        });
    }
    Streamer->SetConnectionURL(Url);
    Streamer->StartStreaming();
    Started = true;
}
bool FPreviewStreaming::Send(const TSharedPtr<FJsonObject>& Message)
{
    if (!Started || !IPixelStreaming2Module::IsAvailable()) return false;
    auto& Module = IPixelStreaming2Module::Get();
    auto Streamer = Module.FindStreamer(Module.GetDefaultStreamerID());
    if (!Streamer.IsValid()) return false;
    FString Json;
    FJsonSerializer::Serialize(Message.ToSharedRef(), TJsonWriterFactory<>::Create(&Json));
    Streamer->SendAllPlayersMessage(TEXT("Response"), Json);
    return true;
}
void FPreviewStreaming::Stop()
{
    if (Started && IPixelStreaming2Module::IsAvailable()) IPixelStreaming2Module::Get().StopStreaming();
    Started = false;
}
