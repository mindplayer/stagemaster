#include "PreviewCameraPawn.h"
#include "PreviewSceneActor.h"
#include "PreviewViewport.h"
#include "Camera/CameraComponent.h"
#include "DrawDebugHelpers.h"
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
    if (ProjectId != Scene->GetScene().ProjectId)
    {
        CancelDrag();
        ProjectId = Scene->GetScene().ProjectId;
        SelectedId.Empty();
        FocusAll();
    }
    if (Dragging && (!Scene->CanMoveFixtures() || DragSerial != Scene->GetSceneSerial()))
    {
        CancelDrag();
        InteractionMessage = TEXT("灯位拖动已取消：连接或工程状态变化");
    }
    if (const auto Fixture = Scene->FindFixture(SelectedId))
    {
        const FVector Position = Dragging && DragMoved ? DragPosition : Fixture->Origin;
        DrawDebugSphere(GetWorld(), Position, 17, 16, FColor(124, 168, 255), false, 0, 0, 1.5f);
        DrawDebugDirectionalArrow(GetWorld(), Position, Position + Fixture->Direction * 80, 10, FColor(124, 168, 255), false, 0, 0, 1.5f);
    }
    else SelectedId.Empty();
}
void APreviewCameraPawn::UpdateCamera()
{
    const FRotator Rotation(Pitch, Yaw, 0);
    SetActorLocationAndRotation(Pivot - Rotation.Vector() * Distance, Rotation);
}
void APreviewCameraPawn::ViewAction(const FString& Action)
{
    if (Action == TEXT("all")) FocusAll();
    else if (Action == TEXT("top")) TopView();
    else if (Action == TEXT("perspective")) { Pitch = -35; Yaw = -45; FocusAll(); }
    else if (Action == TEXT("selected")) FocusSelected();
    else if (Action == TEXT("workLight")) ToggleWorkLight();
    else if (Action == TEXT("cancel")) CancelDrag();
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
    const FBox Bounds = Scene->GetBounds();
    if (!Bounds.IsValid) return;
    Pivot = Bounds.GetCenter();
    int32 Width = 1, Height = 1;
    if (auto Player = Cast<APlayerController>(GetController())) Player->GetViewportSize(Width, Height);
    const double Aspect = FMath::Max(1.0, static_cast<double>(Width) / FMath::Max(1, Height));
    const double HalfAngle = FMath::Atan(FMath::Tan(FMath::DegreesToRadians(27.5)) / Aspect);
    Distance = FMath::Max(100.0, Bounds.GetExtent().Size() / FMath::Sin(HalfAngle) * 1.1);
    UpdateCamera();
}
void APreviewCameraPawn::FocusSelected()
{
    CancelDrag();
    if (Scene) if (const auto Fixture = Scene->FindFixture(SelectedId))
    {
        Pivot = Fixture->Origin;
        Distance = 700;
        UpdateCamera();
    }
}
void APreviewCameraPawn::TopView()
{
    CancelDrag();
    Pitch = -89.9f;
    Yaw = 90;
    FocusAll();
    UpdateCamera();
}
bool APreviewCameraPawn::PointOnDragPlane(const FVector2D& Screen, FVector& Point) const
{
    auto Player = Cast<APlayerController>(GetController());
    FVector Origin, Direction;
    if (!Player || !Player->DeprojectScreenPositionToWorld(static_cast<float>(Screen.X), static_cast<float>(Screen.Y), Origin, Direction) || FMath::Abs(Direction.Z) < 0.0001) return false;
    const double T = (DragOrigin.Z - Origin.Z) / Direction.Z;
    if (T < 0 || T > 20000000) return false;
    Point = Origin + Direction * T;
    return true;
}
void APreviewCameraPawn::SelectAt(const FVector2D& Screen)
{
    CancelDrag();
    InteractionMessage.Empty();
    auto Player = Cast<APlayerController>(GetController());
    if (!Player || !Scene) return;
    FHitResult Hit;
    Player->GetHitResultAtScreenPosition(Screen, ECC_Visibility, true, Hit);
    SelectedId = Scene->FixtureAt(Hit);
    const auto Fixture = Scene->FindFixture(SelectedId);
    if (!MoveMode || !Fixture) return;
    if (!Scene->CanMoveFixtures())
    {
        InteractionMessage = TEXT("请先在舞台大师启用三维灯位编辑");
        return;
    }
    DragOrigin = Fixture->Origin;
    FVector Point;
    if (!PointOnDragPlane(Screen, Point))
    {
        InteractionMessage = TEXT("当前镜头过于平直，请俯视后拖动");
        return;
    }
    DragOffset = DragOrigin - Point;
    DragPosition = DragOrigin;
    DragScreenOrigin = Screen;
    DragSerial = Scene->GetSceneSerial();
    Dragging = true;
    DragMoved = false;
}
void APreviewCameraPawn::DragTo(const FVector2D& Screen)
{
    if (!Dragging || !Scene) return;
    if (!Scene->CanMoveFixtures() || DragSerial != Scene->GetSceneSerial()) { CancelDrag(); return; }
    if (!DragMoved && (Screen - DragScreenOrigin).SizeSquared() < 9) return;
    FVector Point;
    if (PointOnDragPlane(Screen, Point) && Scene->PreviewPosition(SelectedId, Point + DragOffset))
    {
        DragPosition = Point + DragOffset;
        DragMoved = true;
    }
}
void APreviewCameraPawn::FinishDrag()
{
    if (!Dragging || !Scene) return;
    Dragging = false;
    if (DragMoved && !DragPosition.Equals(DragOrigin, 0.00001) && DragSerial == Scene->GetSceneSerial())
        Scene->CommitPosition(SelectedId, DragPosition);
    else Scene->RestorePosition(SelectedId);
    DragMoved = false;
}
void APreviewCameraPawn::CancelDrag()
{
    if (Dragging && Scene) Scene->RestorePosition(SelectedId);
    Dragging = false;
    DragMoved = false;
}
void APreviewCameraPawn::ToggleMove()
{
    CancelDrag();
    MoveMode = !MoveMode;
    InteractionMessage = MoveMode && Scene && !Scene->CanMoveFixtures() ? TEXT("请先在舞台大师启用三维灯位编辑") : FString();
}
void APreviewCameraPawn::ToggleWorkLight() { if (Scene) Scene->ToggleWorkLight(); }
FText APreviewCameraPawn::StatusText() const
{
    const FString Status = Scene ? Scene->GetStatus() : TEXT("正在载入");
    return FText::FromString(InteractionMessage.IsEmpty() ? Status : Status + TEXT("  ·  ") + InteractionMessage);
}
FText APreviewCameraPawn::SelectionText() const
{
    if (Scene) if (const auto Fixture = Scene->FindFixture(SelectedId))
    {
        const auto Position = StageMaster::ToMeters(Dragging && DragMoved ? DragPosition : Fixture->Origin);
        return FText::FromString(FString::Printf(TEXT("%s   位置 %.3f / %.3f / %.3f 米"), *Fixture->Name, Position.X, Position.Y, Position.Z));
    }
    return FText::FromString(TEXT("未选择灯具"));
}
FText APreviewCameraPawn::MoveText() const { return FText::FromString(MoveMode ? TEXT("灯位移动") : TEXT("查看与选择")); }
FText APreviewCameraPawn::WorkLightText() const { return FText::FromString(Scene && Scene->HasWorkLight() ? TEXT("工作照明：开") : TEXT("工作照明：关")); }
