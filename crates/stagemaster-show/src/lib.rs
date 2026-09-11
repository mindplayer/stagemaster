//! Editable show objects. This crate has no UI, storage, network, or device dependency.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use stagemaster_domain::{
    Attribute, AttributeAddress, CueId, FixtureId, GroupId, NormalizedValue, PresetId, SequenceId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixtureGroup {
    pub id: GroupId,
    pub name: String,
    /// Order is semantic: Fan, phase distribution, Next, and Prev consume it.
    pub fixtures: Vec<FixtureId>,
}

impl FixtureGroup {
    #[must_use]
    pub fn new(
        id: GroupId,
        name: impl Into<String>,
        fixtures: impl IntoIterator<Item = FixtureId>,
    ) -> Self {
        let mut seen = BTreeSet::new();
        let fixtures = fixtures
            .into_iter()
            .filter(|fixture| seen.insert(*fixture))
            .collect();
        Self {
            id,
            name: name.into(),
            fixtures,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Preset {
    pub id: PresetId,
    pub name: String,
    pub values: BTreeMap<AttributeAddress, NormalizedValue>,
}

impl Preset {
    #[must_use]
    pub fn selective(
        id: PresetId,
        name: impl Into<String>,
        values: impl IntoIterator<Item = (AttributeAddress, NormalizedValue)>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            values: values.into_iter().collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueSource {
    Literal(NormalizedValue),
    Preset(PresetId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgrammerEntry {
    pub resolved: NormalizedValue,
    pub source: ValueSource,
    pub active: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Programmer {
    selection: Vec<FixtureId>,
    entries: BTreeMap<AttributeAddress, ProgrammerEntry>,
}

impl Programmer {
    pub fn select(&mut self, fixtures: impl IntoIterator<Item = FixtureId>) {
        let mut seen = BTreeSet::new();
        self.selection = fixtures.into_iter().filter(|id| seen.insert(*id)).collect();
    }

    #[must_use]
    pub fn selection(&self) -> &[FixtureId] {
        &self.selection
    }

    #[must_use]
    pub fn entries(&self) -> &BTreeMap<AttributeAddress, ProgrammerEntry> {
        &self.entries
    }

    pub fn set_literal(&mut self, attribute: Attribute, value: NormalizedValue) {
        for fixture in &self.selection {
            self.entries.insert(
                AttributeAddress::new(*fixture, attribute),
                ProgrammerEntry {
                    resolved: value,
                    source: ValueSource::Literal(value),
                    active: true,
                },
            );
        }
    }

    /// Applies every selective preset value belonging to the current selection.
    ///
    /// # Errors
    ///
    /// Returns [`ShowError::PresetDoesNotApply`] when any selected fixture has no value in the
    /// preset. The operation validates each fixture before inserting its matching values.
    pub fn apply_preset(&mut self, preset: &Preset) -> Result<(), ShowError> {
        for fixture in &self.selection {
            if !preset
                .values
                .keys()
                .any(|address| address.fixture == *fixture)
            {
                return Err(ShowError::PresetDoesNotApply {
                    preset: preset.id,
                    fixture: *fixture,
                });
            }
        }
        for fixture in &self.selection {
            let matched: Vec<_> = preset
                .values
                .iter()
                .filter(|(address, _)| address.fixture == *fixture)
                .map(|(address, value)| (*address, *value))
                .collect();
            for (address, value) in matched {
                self.entries.insert(
                    address,
                    ProgrammerEntry {
                        resolved: value,
                        source: ValueSource::Preset(preset.id),
                        active: true,
                    },
                );
            }
        }
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn deactivate_all(&mut self) {
        for entry in self.entries.values_mut() {
            entry.active = false;
        }
    }

    pub fn release_all(&mut self) {
        self.entries.clear();
    }

    #[must_use]
    pub fn record_cue(&self, id: CueId, number: CueNumber, name: impl Into<String>) -> Cue {
        let values = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.active)
            .map(|(address, entry)| (*address, CueValue::Set(entry.source)))
            .collect();
        Cue {
            id,
            number,
            name: name.into(),
            values,
        }
    }
}

/// Cue number in thousandths, so 2.5 and later inserted cues sort without floats.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CueNumber(pub u32);

impl CueNumber {
    #[must_use]
    pub const fn whole(number: u32) -> Self {
        Self(number.saturating_mul(1_000))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CueValue {
    Set(ValueSource),
    Release,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cue {
    pub id: CueId,
    pub number: CueNumber,
    pub name: String,
    pub values: BTreeMap<AttributeAddress, CueValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sequence {
    pub id: SequenceId,
    pub name: String,
    cues: Vec<Cue>,
}

impl Sequence {
    #[must_use]
    pub fn new(id: SequenceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            cues: Vec::new(),
        }
    }

    /// Inserts a new cue or replaces a cue with the same stable ID.
    ///
    /// # Errors
    ///
    /// Returns [`ShowError::DuplicateCueNumber`] when a different cue already uses the number.
    pub fn upsert_cue(&mut self, cue: Cue) -> Result<(), ShowError> {
        if let Some(existing) = self.cues.iter_mut().find(|existing| existing.id == cue.id) {
            *existing = cue;
        } else if self
            .cues
            .iter()
            .any(|existing| existing.number == cue.number)
        {
            return Err(ShowError::DuplicateCueNumber(cue.number));
        } else {
            self.cues.push(cue);
        }
        self.cues.sort_by_key(|cue| cue.number);
        Ok(())
    }

    #[must_use]
    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    /// Resolves sparse cue data from the start of the sequence through `target`.
    ///
    /// # Errors
    ///
    /// Returns an error when the target cue or a referenced preset/value is missing.
    pub fn tracked_state(
        &self,
        target: CueId,
        presets: &BTreeMap<PresetId, Preset>,
    ) -> Result<BTreeMap<AttributeAddress, NormalizedValue>, ShowError> {
        let mut state = BTreeMap::new();
        let mut target_found = false;
        for cue in &self.cues {
            for (address, value) in &cue.values {
                match value {
                    CueValue::Release => {
                        state.remove(address);
                    }
                    CueValue::Set(ValueSource::Literal(value)) => {
                        state.insert(*address, *value);
                    }
                    CueValue::Set(ValueSource::Preset(preset_id)) => {
                        let preset = presets
                            .get(preset_id)
                            .ok_or(ShowError::MissingPreset(*preset_id))?;
                        let value =
                            preset
                                .values
                                .get(address)
                                .ok_or(ShowError::MissingPresetValue {
                                    preset: *preset_id,
                                    address: *address,
                                })?;
                        state.insert(*address, *value);
                    }
                }
            }
            if cue.id == target {
                target_found = true;
                break;
            }
        }
        if !target_found {
            return Err(ShowError::MissingCue(target));
        }
        Ok(state)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowError {
    DuplicateCueNumber(CueNumber),
    MissingCue(CueId),
    MissingPreset(PresetId),
    MissingPresetValue {
        preset: PresetId,
        address: AttributeAddress,
    },
    PresetDoesNotApply {
        preset: PresetId,
        fixture: FixtureId,
    },
}

impl fmt::Display for ShowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ShowError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_preserve_order_and_remove_duplicates() {
        let group = FixtureGroup::new(
            GroupId(1),
            "Front",
            [FixtureId(3), FixtureId(1), FixtureId(3), FixtureId(2)],
        );
        assert_eq!(group.fixtures, [FixtureId(3), FixtureId(1), FixtureId(2)]);
    }

    #[test]
    fn cue_tracks_values_and_keeps_live_preset_reference() {
        let fixture = FixtureId(1);
        let address = AttributeAddress::new(fixture, Attribute::Blue);
        let preset_id = PresetId(7);
        let mut preset = Preset::selective(
            preset_id,
            "Ocean",
            [(address, NormalizedValue::from_percent(80))],
        );
        let mut programmer = Programmer::default();
        programmer.select([fixture]);
        programmer.apply_preset(&preset).expect("preset applies");
        let cue = programmer.record_cue(CueId(1), CueNumber::whole(1), "Blue");
        let mut sequence = Sequence::new(SequenceId(1), "Main");
        sequence.upsert_cue(cue).expect("cue number is unique");

        preset
            .values
            .insert(address, NormalizedValue::from_percent(60));
        let state = sequence
            .tracked_state(CueId(1), &BTreeMap::from([(preset_id, preset)]))
            .expect("reference resolves");
        assert_eq!(state[&address], NormalizedValue::from_percent(60));
    }

    #[test]
    fn deactivated_programmer_values_are_not_recorded() {
        let mut programmer = Programmer::default();
        programmer.select([FixtureId(1)]);
        programmer.set_literal(Attribute::Intensity, NormalizedValue::FULL);
        programmer.deactivate_all();
        let cue = programmer.record_cue(CueId(1), CueNumber::whole(1), "Empty");
        assert!(cue.values.is_empty());
        assert_eq!(programmer.entries().len(), 1);
    }

    #[test]
    fn applying_a_preset_is_atomic_when_one_fixture_is_unsupported() {
        let first = FixtureId(1);
        let second = FixtureId(2);
        let preset = Preset::selective(
            PresetId(1),
            "Only first fixture",
            [(
                AttributeAddress::new(first, Attribute::Blue),
                NormalizedValue::FULL,
            )],
        );
        let mut programmer = Programmer::default();
        programmer.select([first, second]);
        assert!(programmer.apply_preset(&preset).is_err());
        assert!(programmer.entries().is_empty());
    }
}
