using UnrealBuildTool;

public class StageMasterPreview : ModuleRules
{
    public StageMasterPreview(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = PCHUsageMode.UseExplicitOrSharedPCHs;
        PublicDependencyModuleNames.AddRange(new[] { "Core", "CoreUObject", "Engine", "InputCore" });
        PrivateDependencyModuleNames.AddRange(new[] { "HTTP", "Json", "ProceduralMeshComponent", "Slate", "SlateCore", "PixelStreaming2", "PixelStreaming2Core", "PixelStreaming2Input" });
    }
}
