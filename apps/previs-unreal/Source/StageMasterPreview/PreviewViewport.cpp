#include "PreviewViewport.h"
#include "PreviewCameraPawn.h"
#include "GameFramework/PlayerController.h"
#include "Input/Reply.h"
#include "InputCoreTypes.h"
#include "Styling/CoreStyle.h"
#include "Rendering/DrawElements.h"
#include "Widgets/SCompoundWidget.h"
#include "Widgets/SOverlay.h"
#include "Widgets/SBoxPanel.h"
#include "Widgets/Input/SButton.h"
#include "Widgets/Layout/SBorder.h"
#include "Widgets/Text/STextBlock.h"

namespace StageMaster
{
namespace
{
class SPreviewInput : public SCompoundWidget
{
public:
    SLATE_BEGIN_ARGS(SPreviewInput) {} SLATE_ARGUMENT(APreviewCameraPawn*, Camera) SLATE_END_ARGS()
    void Construct(const FArguments& Args)
    {
        Camera = Args._Camera;
        ChildSlot[SNew(SBorder).BorderImage(FCoreStyle::Get().GetBrush("NoBorder"))];
    }
    virtual void Tick(const FGeometry& Geometry, const double Time, const float Delta) override
    {
        SCompoundWidget::Tick(Geometry, Time, Delta);
        FVector2D Start, End;
        bool Removing = false;
        const bool DrawMarquee = Camera.IsValid() && Camera->GetMarquee(Start, End, Removing);
        if (DrawMarquee || HadMarquee) Invalidate(EInvalidateWidgetReason::Paint);
        HadMarquee = DrawMarquee;
    }
    virtual bool SupportsKeyboardFocus() const override { return true; }
    virtual int32 OnPaint(const FPaintArgs& Args, const FGeometry& Geometry, const FSlateRect& Culling,
        FSlateWindowElementList& Elements, int32 Layer, const FWidgetStyle& Style, bool Enabled) const override
    {
        const int32 LastLayer = SCompoundWidget::OnPaint(Args, Geometry, Culling, Elements, Layer, Style, Enabled);
        FVector2D Start, End;
        bool Removing = false;
        if (!Camera.IsValid() || !Camera->GetMarquee(Start, End, Removing)) return LastLayer;
        int32 Width = 0, Height = 0;
        if (auto Player = Cast<APlayerController>(Camera->GetController())) Player->GetViewportSize(Width, Height);
        if (Width <= 0 || Height <= 0) return LastLayer;
        const FVector2D Size = Geometry.GetLocalSize();
        Start *= Size / FVector2D(Width, Height);
        End *= Size / FVector2D(Width, Height);
        const FLinearColor Color = Removing ? FLinearColor(1, 0.55f, 0.35f) : FLinearColor(0.35f, 0.85f, 0.75f);
        TArray<FVector2D> Points{Start, FVector2D(End.X, Start.Y), End, FVector2D(Start.X, End.Y), Start};
        FSlateDrawElement::MakeLines(Elements, LastLayer + 1, Geometry.ToPaintGeometry(), Points,
            ESlateDrawEffect::None, Color, true, 1.5f);
        return LastLayer + 1;
    }
    virtual FReply OnMouseButtonDown(const FGeometry& Geometry, const FPointerEvent& Event) override
    {
        if (!Camera.IsValid()) return FReply::Unhandled();
        const auto Button = Event.GetEffectingButton();
        if (Button != EKeys::LeftMouseButton && Button != EKeys::RightMouseButton) return FReply::Unhandled();
        // The transparent viewport receives events only outside the toolbar, so buttons cannot move fixtures.
        Navigating = Button == EKeys::RightMouseButton || Event.IsAltDown();
        CapturedButton = Button;
        if (Navigating) Camera->CancelDrag();
        else Camera->SelectAt(ScreenPosition(Geometry, Event), Event.IsShiftDown(), Event.IsControlDown() || Event.IsCommandDown());
        return FReply::Handled().CaptureMouse(AsShared()).SetUserFocus(AsShared());
    }
    virtual FReply OnMouseMove(const FGeometry& Geometry, const FPointerEvent& Event) override
    {
        if (!HasMouseCapture() || !Camera.IsValid()) return FReply::Unhandled();
        if (Navigating) Camera->Navigate(Event.GetCursorDelta(), Event.IsShiftDown());
        else Camera->DragTo(ScreenPosition(Geometry, Event));
        return FReply::Handled();
    }
    virtual FReply OnMouseButtonUp(const FGeometry& Geometry, const FPointerEvent& Event) override
    {
        if (!HasMouseCapture() || Event.GetEffectingButton() != CapturedButton) return FReply::Unhandled();
        if (Camera.IsValid() && !Navigating) Camera->FinishDrag();
        Navigating = false;
        return FReply::Handled().ReleaseMouseCapture();
    }
    virtual void OnMouseCaptureLost(const FCaptureLostEvent& Event) override
    {
        if (Camera.IsValid()) Camera->CancelDrag();
        Navigating = false;
        SCompoundWidget::OnMouseCaptureLost(Event);
    }
    virtual FReply OnMouseWheel(const FGeometry& Geometry, const FPointerEvent& Event) override
    {
        if (!Camera.IsValid()) return FReply::Unhandled();
        Camera->Zoom(Event.GetWheelDelta());
        return FReply::Handled();
    }
    virtual FReply OnKeyDown(const FGeometry& Geometry, const FKeyEvent& Event) override
    {
        if (!Camera.IsValid()) return FReply::Unhandled();
        const auto Key = Event.GetKey();
        if (Key == EKeys::Escape) { Camera->CancelDrag(); return FReply::Handled().ReleaseMouseCapture(); }
        if (Key == EKeys::F) Camera->FocusSelected();
        else if (Key == EKeys::Home) Camera->FocusAll();
        else if (Key == EKeys::T) Camera->TopView();
        else return FReply::Unhandled();
        return FReply::Handled();
    }
private:
    FVector2D ScreenPosition(const FGeometry& Geometry, const FPointerEvent& Event) const
    {
        int32 Width = 1, Height = 1;
        if (auto Player = Cast<APlayerController>(Camera->GetController())) Player->GetViewportSize(Width, Height);
        const auto Local = Geometry.AbsoluteToLocal(Event.GetScreenSpacePosition());
        const auto Size = Geometry.GetLocalSize();
        return FVector2D(Local.X * Width / FMath::Max(1.0f, Size.X), Local.Y * Height / FMath::Max(1.0f, Size.Y));
    }
    TWeakObjectPtr<APreviewCameraPawn> Camera;
    FKey CapturedButton;
    bool Navigating = false;
    bool HadMarquee = false;
};

TSharedRef<SWidget> Button(TWeakObjectPtr<APreviewCameraPawn> Camera, const TCHAR* Label, void (APreviewCameraPawn::*Action)())
{
    return SNew(SButton).ContentPadding(FMargin(12, 8)).IsFocusable(false)
        .Text(FText::FromString(Label))
        .OnClicked_Lambda([Camera, Action]() { if (Camera.IsValid()) (Camera.Get()->*Action)(); return FReply::Handled(); });
}
}
TSharedRef<SWidget> MakePreviewViewport(APreviewCameraPawn* Pawn)
{
    if (!FPlatformMisc::GetEnvironmentVariable(TEXT("STAGEMASTER_STREAM_URL")).IsEmpty())
        return SNew(SPreviewInput).Camera(Pawn);
    const TWeakObjectPtr<APreviewCameraPawn> Camera(Pawn);
    return SNew(SOverlay)
        + SOverlay::Slot()[SNew(SPreviewInput).Camera(Pawn)]
        + SOverlay::Slot().VAlign(VAlign_Top)
        [
            SNew(SBorder).Padding(FMargin(12, 8)).BorderBackgroundColor(FLinearColor(0.02, 0.025, 0.035, 0.96))
            [
                SNew(SHorizontalBox)
                + SHorizontalBox::Slot().AutoWidth().VAlign(VAlign_Center).Padding(0, 0, 20, 0)
                [SNew(STextBlock).Text(FText::FromString(TEXT("舞台大师 · 三维预演"))).Font(FCoreStyle::GetDefaultFontStyle("Bold", 16))]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [Button(Camera, TEXT("全场"), &APreviewCameraPawn::FocusAll)]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [Button(Camera, TEXT("俯视"), &APreviewCameraPawn::TopView)]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [Button(Camera, TEXT("聚焦所选"), &APreviewCameraPawn::FocusSelected)]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [SNew(SButton).ContentPadding(FMargin(12, 8)).IsFocusable(false)
                    .Text_Lambda([Camera]() { return Camera.IsValid() ? Camera->MoveText() : FText(); })
                    .OnClicked_Lambda([Camera]() { if (Camera.IsValid()) Camera->ToggleMove(); return FReply::Handled(); })]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [SNew(SButton).ContentPadding(FMargin(12, 8)).IsFocusable(false)
                    .Text_Lambda([Camera]() { return FText::FromString(Camera.IsValid() && Camera->IsSelectionThrough() ? TEXT("穿透框选：开") : TEXT("穿透框选：关")); })
                    .OnClicked_Lambda([Camera]() { if (Camera.IsValid()) Camera->ToggleSelectionThrough(); return FReply::Handled(); })]
                + SHorizontalBox::Slot().AutoWidth().Padding(2, 0)
                [SNew(SButton).ContentPadding(FMargin(12, 8)).IsFocusable(false)
                    .Text_Lambda([Camera]() { return Camera.IsValid() ? Camera->WorkLightText() : FText(); })
                    .OnClicked_Lambda([Camera]() { if (Camera.IsValid()) Camera->ToggleWorkLight(); return FReply::Handled(); })]
                + SHorizontalBox::Slot().FillWidth(1).HAlign(HAlign_Right).VAlign(VAlign_Center)
                [SNew(STextBlock).Text(FText::FromString(TEXT("通用光学视效"))).ColorAndOpacity(FLinearColor(0.55, 0.6, 0.7))]
            ]
        ]
        + SOverlay::Slot().VAlign(VAlign_Bottom)
        [
            SNew(SBorder).Padding(FMargin(16, 10)).BorderBackgroundColor(FLinearColor(0.02, 0.025, 0.035, 0.96))
            [
                SNew(SVerticalBox)
                + SVerticalBox::Slot().AutoHeight()
                [SNew(STextBlock).Text_Lambda([Camera]() { return Camera.IsValid() ? Camera->StatusText() : FText(); })]
                + SVerticalBox::Slot().AutoHeight().Padding(0, 4)
                [SNew(STextBlock).Text_Lambda([Camera]() { return Camera.IsValid() ? Camera->SelectionText() : FText(); })]
                + SVerticalBox::Slot().AutoHeight()
                [SNew(STextBlock).Text(FText::FromString(TEXT("拖框选灯 · Shift 加选 · Ctrl / Command 减选 · Option 旋转 · 滚动缩放 · Esc 取消")))
                    .ColorAndOpacity(FLinearColor(0.55, 0.6, 0.7))]
            ]
        ];
}
}
