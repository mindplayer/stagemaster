using UnrealBuildTool;

public class StageMasterPreviewEditorTarget : TargetRules
{
    public StageMasterPreviewEditorTarget(TargetInfo Target) : base(Target)
    {
        Type = TargetType.Editor;
        DefaultBuildSettings = BuildSettingsVersion.V7;
        IncludeOrderVersion = EngineIncludeOrderVersion.Unreal5_8;
        ExtraModuleNames.Add("StageMasterPreview");
    }
}
