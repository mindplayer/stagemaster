use std::collections::BTreeMap;

use stagemaster_domain::{
    Attribute, AttributeAddress, CueId, FixtureId, NormalizedValue, SequenceId,
};
use stagemaster_show::{Cue, CueNumber, CueValue, Sequence, ShowError, ValueSource};

fn cue(id: u64, number: CueNumber, name: &str, value: u16) -> Cue {
    Cue {
        id: CueId(id),
        number,
        name: name.to_owned(),
        values: BTreeMap::from([(
            AttributeAddress::new(FixtureId(id), Attribute::Intensity),
            CueValue::Set(ValueSource::Literal(NormalizedValue::from_raw(value))),
        )]),
    }
}

#[test]
fn inserts_new_cue_at_free_number_and_sorts() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    sequence
        .upsert_cue(cue(2, CueNumber::whole(2), "Second", 2))
        .unwrap();
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "First", 1))
        .unwrap();
    assert_eq!(
        sequence.cues().iter().map(|cue| cue.id).collect::<Vec<_>>(),
        [CueId(1), CueId(2)]
    );
}

#[test]
fn rejects_new_id_collision_without_mutation() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "Original", 100))
        .unwrap();
    let before = sequence.clone();
    assert_eq!(
        sequence.upsert_cue(cue(2, CueNumber::whole(1), "Collision", 200)),
        Err(ShowError::DuplicateCueNumber(CueNumber::whole(1)))
    );
    assert_eq!(sequence, before);
}

#[test]
fn rejects_existing_id_moving_to_another_cues_number_without_mutation() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "First", 100))
        .unwrap();
    sequence
        .upsert_cue(cue(2, CueNumber::whole(2), "Second", 200))
        .unwrap();
    let before = sequence.clone();

    assert_eq!(
        sequence.upsert_cue(cue(1, CueNumber::whole(2), "Illegal replacement", 999)),
        Err(ShowError::DuplicateCueNumber(CueNumber::whole(2)))
    );
    assert_eq!(sequence, before);
}

#[test]
fn replaces_same_id_at_its_own_number_without_growing() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "Before", 100))
        .unwrap();
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "After", 200))
        .unwrap();

    assert_eq!(sequence.cues().len(), 1);
    assert_eq!(
        sequence.cues()[0],
        cue(1, CueNumber::whole(1), "After", 200)
    );
}

#[test]
fn moves_existing_id_to_free_smaller_and_larger_numbers_in_order() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    sequence
        .upsert_cue(cue(1, CueNumber::whole(1), "First", 1))
        .unwrap();
    sequence
        .upsert_cue(cue(2, CueNumber::whole(2), "Moving", 2))
        .unwrap();
    sequence
        .upsert_cue(cue(3, CueNumber::whole(3), "Third", 3))
        .unwrap();

    sequence
        .upsert_cue(cue(2, CueNumber(500), "Smaller", 20))
        .unwrap();
    assert_eq!(
        sequence.cues().iter().map(|cue| cue.id).collect::<Vec<_>>(),
        [CueId(2), CueId(1), CueId(3)]
    );

    sequence
        .upsert_cue(cue(2, CueNumber::whole(4), "Larger", 40))
        .unwrap();
    assert_eq!(
        sequence.cues().iter().map(|cue| cue.id).collect::<Vec<_>>(),
        [CueId(1), CueId(3), CueId(2)]
    );
    assert_eq!(sequence.cues().len(), 3);
}

#[test]
fn repeated_identical_cue_stays_unique() {
    let mut sequence = Sequence::new(SequenceId(1), "Main");
    let same = cue(1, CueNumber::whole(1), "Same", 100);
    sequence.upsert_cue(same.clone()).unwrap();
    sequence.upsert_cue(same.clone()).unwrap();
    assert_eq!(sequence.cues(), [same]);
}
