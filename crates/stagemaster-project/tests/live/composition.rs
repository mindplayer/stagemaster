use super::support::*;
use stagemaster_engine::live::{Frame, Kind, LiveMixer};
use stagemaster_playback::Player;

#[test]
fn real_dynamic_scenes_and_manual_source_mix_sparse_ownership_then_return_to_running_values() {
    let (doc, _, ids) = setup();
    let mut a = doc.compile_live_scene(&ids[0], 0).unwrap();
    let mut b = doc.compile_live_scene(&ids[1], 0).unwrap();
    let mut reference = Player::new(doc.compile_scene(&ids[0]).unwrap().plan, 0);
    reference.execute(0, 0).unwrap();
    let mut mixer = LiveMixer::new([7; 16], a.layout().clone(), 3).unwrap();
    let ah = open(&mut mixer, 1, Kind::Playback, 0);
    let bh = open(&mut mixer, 2, Kind::Playback, 0);
    a.start(0).unwrap();
    a.publish(&mut mixer, ah, 1).unwrap();
    b.start(0).unwrap();
    b.publish(&mut mixer, bh, 1).unwrap();
    advance(&mut a, &mut mixer, ah, 2, 250);
    advance(&mut b, &mut mixer, bh, 2, 250);
    let sampled = values(&mixer);
    assert_eq!(sampled[1], 0x1fff, "routine sampling cannot take LTP back");
    assert_eq!(
        sampled[2], 22_000,
        "unowned red default cannot cover the background"
    );
    assert_eq!(
        sampled[3],
        20 * 257,
        "unowned wheel default cannot change its slot"
    );
    mixer.set_level(bh, 3, 0).unwrap();
    advance(&mut a, &mut mixer, ah, 3, 500);
    advance(&mut b, &mut mixer, bh, 4, 500);
    reference.advance(500).unwrap();
    assert_eq!(
        values(&mixer),
        [reference.values()[0], 0x1fff, 22_000, 20 * 257]
    );
    super::port::verify(&a, &mixer);
    let hand = open(&mut mixer, 3, Kind::Programmer, 10);
    let identity = mixer.layout().id();
    mixer
        .publish(
            hand,
            Frame {
                layout: identity,
                serial: 1,
                values: &[Some(0), None, None, Some(40 * 257), None, None, None],
                assert: &[false; 7],
            },
        )
        .unwrap();
    assert_eq!(values(&mixer), [0, 0x1fff, 22_000, 40 * 257]);
    mixer
        .publish(
            hand,
            Frame {
                layout: identity,
                serial: 2,
                values: &[Some(0), None, None, None, None, None, None],
                assert: &[false; 7],
            },
        )
        .unwrap();
    assert_eq!(values(&mixer)[3], 20 * 257);
    mixer.close(hand, 3).unwrap();
    b.stop(750).unwrap();
    b.publish(&mut mixer, bh, 5).unwrap();
    advance(&mut a, &mut mixer, ah, 4, 750);
    reference.advance(750).unwrap();
    assert_eq!(values(&mixer), reference.values()[..4]);
    assert_ne!(values(&mixer)[1], 0x1fff);
    a.stop(1000).unwrap();
    a.publish(&mut mixer, ah, 5).unwrap();
    assert_eq!(values(&mixer), [12000, 32768, 0, 0]);
}
