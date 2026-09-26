#include "StageMasterGameMode.h"
#include "PreviewCameraPawn.h"

AStageMasterGameMode::AStageMasterGameMode()
{
    DefaultPawnClass = APreviewCameraPawn::StaticClass();
}
