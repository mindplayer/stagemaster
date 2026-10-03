#include "PreviewMarquee.h"
#if WITH_DEV_AUTOMATION_TESTS
#include "Misc/AutomationTest.h"

IMPLEMENT_SIMPLE_AUTOMATION_TEST(FPreviewMarqueeTest, "StageMaster.Previs.MarqueeSelection",
    EAutomationTestFlags::EditorContext | EAutomationTestFlags::EngineFilter)
bool FPreviewMarqueeTest::RunTest(const FString& Parameters)
{
    using namespace StageMaster;
    const FVector2D Size(800, 600), A(100, 200), B(300, 400);
    for (const auto& Point : {A, B, FVector2D(150, 250)})
    {
        TestTrue(TEXT("inclusive point containment"), InsideMarquee(Point, A, B, Size));
        TestTrue(TEXT("reverse drag identical region"), InsideMarquee(Point, B, A, Size));
    }
    for (const auto& Point : {FVector2D(99, 250), FVector2D(250, 401), FVector2D(400, 250)})
        TestFalse(TEXT("outside rectangle rejected"), InsideMarquee(Point, A, B, Size));
    TestFalse(TEXT("outside viewport rejected even if rectangle extends outside"), InsideMarquee(FVector2D(-1, 200), FVector2D(-100, 0), B, Size));
    TestFalse(TEXT("zero viewport rejected"), InsideMarquee(A, A, B, FVector2D::ZeroVector));
    TArray<FString> Result{TEXT("unchanged")};
    const TArray<FString> Before{TEXT("c"), TEXT("a")}, Hits{TEXT("a"), TEXT("b")};
    TestTrue(TEXT("replace succeeds"), MarqueeSelection(Before, Hits, EMarqueeMode::Replace, Result));
    TestTrue(TEXT("replace follows scene order"), Result == Hits);
    TestTrue(TEXT("add succeeds"), MarqueeSelection(Before, Hits, EMarqueeMode::Add, Result));
    TestTrue(TEXT("append preserves original ordering without duplicates"), Result == TArray<FString>({TEXT("c"), TEXT("a"), TEXT("b")}));
    TestTrue(TEXT("remove succeeds"), MarqueeSelection(Before, Hits, EMarqueeMode::Remove, Result));
    TestTrue(TEXT("unhit members preserved"), Result == TArray<FString>{TEXT("c")});
    TestTrue(TEXT("blank add preserves"), MarqueeSelection(Before, {}, EMarqueeMode::Add, Result) && Result == Before);
    TestTrue(TEXT("blank remove preserves"), MarqueeSelection(Before, {}, EMarqueeMode::Remove, Result) && Result == Before);
    TestTrue(TEXT("blank replace clears"), MarqueeSelection(Before, {}, EMarqueeMode::Replace, Result) && Result.IsEmpty());
    TArray<FString> Large;
    for (int32 I = 0; I < 1024; ++I) Large.Add(FString::FromInt(I));
    TestTrue(TEXT("full stage accepted"), MarqueeSelection({}, Large, EMarqueeMode::Replace, Result) && Result == Large);
    TestFalse(TEXT("overflow not truncated"), MarqueeSelection(Large, {TEXT("extra")}, EMarqueeMode::Add, Result));
    TestTrue(TEXT("failure preserves prior result"), Result == Large);
    TestFalse(TEXT("duplicate rejected"), MarqueeSelection({}, {TEXT("a"), TEXT("a")}, EMarqueeMode::Replace, Result));
    FMarqueeGesture Gesture;
    Gesture.Begin(A, 7, true, false);
    Gesture.Update(A + FVector2D(2, 0));
    TestTrue(TEXT("click threshold retains additive click"), Gesture.Active && !Gesture.Moved && Gesture.ClickAdditive);
    Gesture.Update(B);
    TestTrue(TEXT("rectangle starts after threshold"), Gesture.Moved && Gesture.Mode == EMarqueeMode::Add && Gesture.Serial == 7);
    Gesture.Update(A);
    TestTrue(TEXT("return to start remains rectangle rather than unexpected click"), Gesture.Moved);
    Gesture.Cancel();
    Gesture.Update(B);
    TestFalse(TEXT("late move cannot revive cancellation"), Gesture.Active || Gesture.Moved);
    Gesture.Begin(B, 8, true, true);
    TestTrue(TEXT("remove modifier takes priority"), Gesture.Mode == EMarqueeMode::Remove && Gesture.Serial == 8 && !Gesture.Moved);
    Gesture.Begin(A, 9, false, false, EMarqueeMode::Remove);
    TestTrue(TEXT("explicit mode supports unmodified drag"), Gesture.Mode == EMarqueeMode::Remove && !Gesture.ClickAdditive);
    Gesture.Begin(A, 9, true, false, EMarqueeMode::Remove);
    TestTrue(TEXT("held modifier overrides explicit mode"), Gesture.Mode == EMarqueeMode::Add);
    return true;
}
#endif
