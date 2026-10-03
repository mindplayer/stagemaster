//! Authoritative, UI-independent editor for the supported lighting project subset.
mod live_audio;
pub use live_audio::{LiveAudioTimeline, LiveSourceBudget};
mod audio_crossfade;
mod audio_crossfade_compile;
mod audio_crossfade_validate;
mod audio_segment;
pub use audio_crossfade::{ClipCrossfadeSource, ClipEntryCrossfade, ClipFadeMode};
pub use audio_segment::{AudioSegmentPlan, AudioSegmentPlayer, CompiledAudioSegment};
mod audio;
mod audio_loop_edit;
mod audio_loops;
pub use audio_loop_edit::{AudioLoopEdit, AudioLoopGroupAction};
pub use audio_loops::{
    AudioLoopPlays, AudioLoopRegion, CompiledAudioLoops, MAX_AUDIO_LOOP_REGIONS,
};
mod audio_clip_compile;
mod audio_clip_edit;
mod audio_clip_fade;
pub use audio_clip_fade::{ClipEntryFade, ClipFadeValue};
mod audio_clips;
pub use audio_clips::{AudioLightingClip, AudioLightingRef, MAX_LIGHTING_CLIPS};
mod audio_clip_group;
mod audio_clip_offset;
mod audio_clip_split;
mod audio_clip_state;
mod audio_group;
pub use audio_clip_group::LightingClipGroupAction;
mod audio_lighting;
pub use audio::{
    AudioAsset, AudioEdit, AudioMarker, AudioTimeline, MAX_AUDIO_MARKERS, MAX_AUDIO_MS,
};
mod check;
mod compilation;
mod live_output;
mod live_player;
mod live_scene;
mod live_sequence;
pub use live_output::LiveOutput;
pub use live_player::LiveScenePlayer;
pub use live_sequence::LiveSequencePlayer;
mod package;
mod patch_report;
mod report_csv;
mod sequence_report;
pub use check::{
    CheckIssue, CheckLocation, CheckReport, PlanLimits, PlanUsage, ProgramCheck, ProgramStatus,
    Severity,
};
pub use package::{PackageBuild, PackageIssue, PackageProgram, PackageReport, PackageSelection};
pub use patch_report::PatchReport;
pub use sequence_report::SequenceReport;
mod editing;
mod effect_compile;
mod effect_template;
pub use effect_template::{
    EffectTemplate, EffectTemplateDefinition, EffectTemplateFile, EffectTemplateRecipe,
    EffectTemplateReview, EffectTemplateReviewView, EffectTemplateSource, EffectTemplateTiming,
    IntensityWaveform, MAX_EFFECT_TEMPLATE_BYTES,
};
mod effects;
mod encoding;
mod fixture;
mod fixture_optics;
pub use fixture_optics::is_continuous_optics_attribute;
mod fixture_appearance;
mod fixture_exchange;
mod fixture_function;
mod fixture_function_mapping;
pub use fixture_appearance::WheelAppearance;
mod fixture_value;
mod fixture_view;
mod profile_file;
pub use profile_file::{MAX_PROFILE_FILE_BYTES, ProfileFile, ProfileSource};
mod function_output;
mod output;
mod output_intensity;
pub use fixture_function::{
    FunctionDefinition, FunctionMode, FunctionSelection, FunctionTable, MAX_CHANNEL_FUNCTIONS,
    ProfileDefault,
};
pub use function_output::FunctionOutput;
mod position;
mod position_effect;
mod world_line;
mod world_line_curve;
pub use effects::{EffectEdit, EffectKeyframe, EffectValues, SceneEffect, Transition, Waveform};
pub use fixture::{FixtureEdit, ProfileChannel, ProfileDefinition, Repatch};
pub use position::{FixtureZero, PositionAxis, PositionEdit, PositionModel};
pub use position::{
    PositionReference, PositionReferenceView, ReferenceCheckView, ReferencePoint,
    ReferencePointView, ReferenceSource,
};
pub use world_line::{TargetBranch, TargetPath};
mod library;
mod rigging;
mod seating;
mod sequence;
mod sequence_groups;
pub use sequence_groups::StepGroupOperation;
mod sequence_script;
mod sequence_script_patch;
pub use sequence_script_patch::StepScriptPatch;
mod sequence_timing;
pub use sequence_timing::{StepAdvance, StepTimingPatch};
mod stage;
mod stage_constructions;
pub use seating::{SeatingAisle, SeatingArc, SeatingLayout, SeatingShape};
mod stage_locks;
mod stage_object_translation;
mod stage_transform;
mod stage_translation;
pub use audio_group::MarkerGroupAction;
pub use compilation::{CompiledSequence, CompiledStep};
pub use output::{AttributeOutput, CompiledOutput, FixtureOutput, OutputObserver, PreviewOutput};
pub use rigging::{RigAttachment, RigKind, RigLayout, RigShape};
pub use sequence::{Repeat, SequenceEdit, Tracking};
pub use sequence_script::StepScript;
pub use stage::{
    ConstructionShape, FixturePlacement, SpatialVector3, StageConstruction, StageEdit, StageSpace,
    StageView,
};
pub use stage_locks::{StageEditLock, StageLockKind};
mod schema;
mod strict_json;
mod validation;
mod view;

pub use editing::{EditCommand, ValueMode};
pub use library::{LibraryEdit, LibraryKind, PresetUpdate};
use serde_json::{Value, json};
use uuid::Uuid;
pub use view::{
    AttributeView, FixtureView, GroupView, NamedView, PresetView, ProfileView, ProjectView,
    SceneValue, SceneView, SequenceView, StepView,
};

/// Maximum input/output bytes; editable compact content must also leave room
/// for the next saved revision (38 bytes when the parent revision list is empty).
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
const VERSION: &str = "0.1.0-draft.1";

#[derive(Clone, Debug)]
pub struct Document {
    root: Value,
    saved_revision: Option<String>,
}
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl Document {
    /// Read and validate an editable lighting project, with no file or device access.
    /// # Errors
    /// Rejects malformed JSON, unsupported capabilities and invalid domain references.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let root = strict_json::decode(bytes)?;
        validation::validate(&root)?;
        encoding::validate_capacity(&root)?;
        Ok(Self {
            saved_revision: Some(text(&root["project"], "revisionId").into()),
            root,
        })
    }
    /// Create an empty lighting project with usable generic fixture definitions.
    /// # Errors
    /// Returns an error if the name is invalid.
    pub fn new(name: &str) -> Result<Self, String> {
        let root = json!({
            "format":"stagemaster.project", "formatVersion":VERSION, "semanticsVersion":VERSION,
            "project":{"id":id(),"name":name,"revisionId":id(),"parentRevisionIds":[],"description":""},
            "requires":[{"key":"lighting.basic","version":1}], "resources":[],
            "domains":[{"id":id(),"name":"灯光","kind":"lighting"}],
            "lighting":{"profiles":[generic_profile(false),generic_profile(true)],"fixtures":[],"groups":[],"presets":[],"scenes":[],"sequences":[],"patches":[]},
            "syncGroups":[],"actions":[],"conditions":[],"rules":[],"timelines":[],"entryPoints":[],"extensions":[]
        });
        validation::validate(&root)?;
        encoding::validate_capacity(&root)?;
        Ok(Self {
            root,
            saved_revision: None,
        })
    }
    #[must_use]
    pub fn view(&self) -> ProjectView {
        view::project(&self.root)
    }
    /// Apply one atomic edit. Rejected edits leave the document unchanged.
    /// # Errors
    /// Returns validation errors or an error for an unknown target.
    pub fn edit(&mut self, command: EditCommand) -> Result<(), String> {
        let mut next = self.root.clone();
        editing::apply(&mut next, command)?;
        validation::validate(&next)?;
        encoding::validate_capacity(&next)?;
        self.root = next;
        Ok(())
    }
    /// Produce a new immutable saved revision without mutating the editing document.
    #[must_use]
    pub fn next_revision(&self) -> Self {
        let mut next = self.clone();
        next.root["project"]["parentRevisionIds"] =
            json!(self.saved_revision.iter().collect::<Vec<_>>());
        next.root["project"]["revisionId"] = id().into();
        next.saved_revision = Some(text(&next.root["project"], "revisionId").into());
        next
    }
    /// Compare editable content while ignoring save revision bookkeeping.
    #[must_use]
    pub fn same_content(&self, other: &Self) -> bool {
        let mut left = self.root.clone();
        let mut right = other.root.clone();
        for root in [&mut left, &mut right] {
            root["project"]["revisionId"] = Value::Null;
            root["project"]["parentRevisionIds"] = Value::Null;
        }
        left == right
    }
    /// Serialize the complete validated document, using compact JSON if indented
    /// JSON would exceed the file limit. No content is omitted to make it fit.
    /// # Errors
    /// Returns an encoding or document-size error.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        encoding::encode(&self.root)
    }
    /// Retain the latest saved revision while moving through local undo history.
    pub fn use_revision_from(&mut self, saved: &Self) {
        self.saved_revision.clone_from(&saved.saved_revision);
        self.root["project"]["revisionId"] = saved.root["project"]["revisionId"].clone();
        self.root["project"]["parentRevisionIds"] =
            saved.root["project"]["parentRevisionIds"].clone();
    }
}
fn id() -> String {
    Uuid::new_v4().to_string()
}
fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}
fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map_or(&[], Vec::as_slice)
}
fn generic_profile(rgb: bool) -> Value {
    let keys = if rgb {
        vec!["dimmer", "red", "green", "blue"]
    } else {
        vec!["dimmer"]
    };
    json!({"id":id(),"revision":id(),"name":if rgb {"通用 RGB 调光 · 4 通道"} else {"通用调光 · 1 通道"},
        "manufacturer":"通用","model":if rgb {"RGBD"} else {"Dimmer"},"mode":if rgb {"4 通道"} else {"1 通道"},"footprint":keys.len(),
        "attributes":keys.iter().map(|key| json!({"key":key,"valueType":{"kind":"normalized"},"default":{"kind":"normalized","value":0},"mix":if *key=="dimmer" {"htp"} else {"ltp"}})).collect::<Vec<_>>(),
        "channels":keys.iter().enumerate().map(|(offset,key)|json!({"attribute":key,"encoding":"u8","offsets":[offset]})).collect::<Vec<_>>()})
}
