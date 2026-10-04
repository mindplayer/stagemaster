#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewViewport.h"
#include "Camera/CameraComponent.h"
#include "Engine/GameViewportClient.h"
#include "Engine/World.h"
#include "GameFramework/PlayerController.h"
#include "Widgets/SWindow.h"

APreviewCameraPawn::APreviewCameraPawn()
{
    PrimaryActorTick.bCanEverTick = true;
    Camera = CreateDefaultSubobject<UCameraComponent>(TEXT("Camera"));
    RootComponent = Camera;
    Camera->SetFieldOfView(55);
    Camera->PostProcessSettings.bOverride_AutoExposureBias = true;
    Camera->PostProcessSettings.AutoExposureBias = -3;
}
void APreviewCameraPawn::BeginPlay()
{
    Super::BeginPlay();
    Scene = GetWorld()->SpawnActor<APreviewSceneActor>();
    Overlay = StageMaster::MakePreviewViewport(this);
    if (auto Viewport = GetWorld()->GetGameViewport())
    {
        Viewport->AddViewportWidgetContent(Overlay.ToSharedRef(), 10);
        if (auto Window = Viewport->GetWindow()) Window->SetTitle(FText::FromString(TEXT("舞台大师 · 三维预演")));
    }
    if (auto Player = Cast<APlayerController>(GetController()))
    {
        Player->bShowMouseCursor = true;
        FInputModeGameAndUI Input;
        Input.SetWidgetToFocus(Overlay);
        Input.SetLockMouseToViewportBehavior(EMouseLockMode::DoNotLock);
        Input.SetHideCursorDuringCapture(false);
        Player->SetInputMode(Input);
    }
    UpdateCamera();
}
void APreviewCameraPawn::EndPlay(const EEndPlayReason::Type Reason)
{
    Streaming.Stop();
    CancelDrag();
    ClearPendingPlacement();
    if (Overlay.IsValid())
    {
        if (auto Viewport = GetWorld()->GetGameViewport()) Viewport->RemoveViewportWidgetContent(Overlay.ToSharedRef());
        Overlay.Reset();
    }
    Super::EndPlay(Reason);
}
void APreviewCameraPawn::Tick(float DeltaSeconds)
{
    Super::Tick(DeltaSeconds);
    Streaming.Tick(this);
    if (!Scene) return;
    TickSelection();
}
void APreviewCameraPawn::UpdateCamera()
{
    InteractionMessage.Empty();
    const FRotator Rotation(Pitch, Yaw, 0);
    SetActorLocationAndRotation(Pivot - Rotation.Vector() * Distance, Rotation);
}
void APreviewCameraPawn::ViewAction(const FString& Action)
{
    if (Action == TEXT("all")) FocusAll();
    else if (Action == TEXT("top")) TopView();
    else if (Action == TEXT("perspective")) { Pitch = -35; Yaw = -45; FocusAll(); }
    else if (Action == TEXT("selected")) FocusSelected();
    else if (Action == TEXT("cutaway")) { CancelDrag(); if (Scene) Scene->ToggleCutaway(); }
    else if (Action == TEXT("workLight")) ToggleWorkLight();
    else if (Action == TEXT("marqueeReplace") || Action == TEXT("marqueeAdd") || Action == TEXT("marqueeRemove"))
    {
        CancelDrag();
        MarqueeMode = Action == TEXT("marqueeAdd") ? StageMaster::EMarqueeMode::Add :
            Action == TEXT("marqueeRemove") ? StageMaster::EMarqueeMode::Remove : StageMaster::EMarqueeMode::Replace;
    }
    else if (Action == TEXT("selectAllObjects") || Action == TEXT("selectFixturesOnly"))
    { CancelDrag(); AllObjects = ObjectsEnabled && Action == TEXT("selectAllObjects"); }
    else if (Action == TEXT("selectionThrough")) ToggleSelectionThrough();
    else if (Action == TEXT("cancel")) CancelDrag();
    else if (Action == TEXT("move")) { CancelDrag(); MoveMode = true; }
    else if (Action == TEXT("inspect")) { CancelDrag(); ClearPendingPlacement(); MoveMode = false; }
    else if (Action == TEXT("moveHorizontal")) { CancelDrag(); Tool = TEXT("horizontal"); }
    else if (Action == TEXT("rotate") || Action == TEXT("scale")) { CancelDrag(); Tool = Action; }
    else if (Action == TEXT("moveVertical")) { CancelDrag(); Tool = TEXT("vertical"); }
}
void APreviewCameraPawn::Navigate(const FVector2D& Delta, bool Pan)
{
    CancelDrag();
    if (Pan)
    {
        const FRotationMatrix Rotation(FRotator(Pitch, Yaw, 0));
        Pivot += (Rotation.GetUnitAxis(EAxis::Y) * -Delta.X + Rotation.GetUnitAxis(EAxis::Z) * Delta.Y) * Distance * 0.0015;
    }
    else
    {
        Yaw -= static_cast<float>(Delta.X * 0.25);
        Pitch = FMath::Clamp(Pitch - static_cast<float>(Delta.Y * 0.25), -89.0f, 89.0f);
    }
    UpdateCamera();
}
void APreviewCameraPawn::Zoom(float Steps)
{
    CancelDrag();
    Distance = FMath::Clamp(Distance * FMath::Exp(-static_cast<double>(Steps) * 0.15), 10.0, 20000000.0);
    UpdateCamera();
}
void APreviewCameraPawn::FocusAll()
{
    CancelDrag();
    if (!Scene) return;
    FitBounds(Scene->GetBounds(), 100);
}
void APreviewCameraPawn::FitBounds(const FBox& Bounds, double MinimumDistance)
{
    if (!Bounds.IsValid) return;
    Pivot = Bounds.GetCenter();
    int32 Width = 1, Height = 1;
    if (auto Player = Cast<APlayerController>(GetController())) Player->GetViewportSize(Width, Height);
    const double Aspect = FMath::Max(1.0, static_cast<double>(Width) / FMath::Max(1, Height));
    const double HalfAngle = FMath::Atan(FMath::Tan(FMath::DegreesToRadians(27.5)) / Aspect);
    Distance = FMath::Max(MinimumDistance, Bounds.GetExtent().Size() / FMath::Sin(HalfAngle) * 1.1);
    UpdateCamera();
}
void APreviewCameraPawn::FocusSelected()
{
    CancelDrag();
    if (!Scene) return;
    FBox Bounds(ForceInit);
    for (const auto& Id : SelectedIds) { const auto Box = Scene->ObjectBounds(Id); if (Box.IsValid) Bounds += Box; }
    if (Bounds.IsValid) FitBounds(Bounds.ExpandBy(30), 300);
}
void APreviewCameraPawn::TopView()
{
    CancelDrag();
    Pitch = -89.9f;
    Yaw = -90;
    FocusAll();
    UpdateCamera();
}
void APreviewCameraPawn::ToggleMove()
{
    CancelDrag();
    MoveMode = !MoveMode;
    InteractionMessage = MoveMode && Scene && !Scene->CanMoveFixtures() ? TEXT("场地尚未同步，请稍后再移动灯位") : FString();
}
void APreviewCameraPawn::ToggleWorkLight() { if (Scene) Scene->ToggleWorkLight(); }
FText APreviewCameraPawn::StatusText() const
{
    const FString Status = Scene ? Scene->GetStatus() : TEXT("正在载入");
    return FText::FromString(InteractionMessage.IsEmpty() ? Status : Status + TEXT("  ·  ") + InteractionMessage);
}
FText APreviewCameraPawn::MoveText() const { return FText::FromString(MoveMode ? TEXT("灯位移动") : TEXT("查看与选择")); }
FText APreviewCameraPawn::WorkLightText() const { return FText::FromString(Scene && Scene->HasWorkLight() ? TEXT("工作照明：开") : TEXT("工作照明：关")); }

bool APreviewCameraPawn::IsCutaway() const { return Scene && Scene->IsCutaway(); }
