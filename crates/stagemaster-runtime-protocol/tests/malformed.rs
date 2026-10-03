mod support;
use minicbor::{Encoder, encode::write::Cursor};
use stagemaster_runtime_protocol::{Body, Error, Operation, Response, Step, Text};

// Construct adversarial CBOR independently of the validated production model/encoder.
fn page(name: &[u8], text_header: &[u8], step_tag: u8) -> Vec<u8> {
    let mut bytes = [0; 1280];
    bytes[..8].copy_from_slice(b"SMRT\x01\x03\0\0");
    let mut e = Encoder::new(Cursor::new(&mut bytes[8..]));
    e.array(3)
        .unwrap()
        .array(4)
        .unwrap()
        .bytes(&[3; 16])
        .unwrap()
        .u64(1)
        .unwrap()
        .u64(0)
        .unwrap()
        .array(2)
        .unwrap()
        .u8(2)
        .unwrap()
        .u16(0)
        .unwrap();
    e.array(5)
        .unwrap()
        .bytes(&[2; 16])
        .unwrap()
        .u64(0)
        .unwrap()
        .u64(0)
        .unwrap()
        .u16(0)
        .unwrap()
        .u16(1)
        .unwrap();
    e.array(2)
        .unwrap()
        .u8(step_tag)
        .unwrap()
        .array(3)
        .unwrap()
        .bytes(&[7; 16])
        .unwrap();
    let mut n = 8 + e.writer().position();
    bytes[n..n + text_header.len()].copy_from_slice(text_header);
    n += text_header.len();
    bytes[n..n + name.len()].copy_from_slice(name);
    n += name.len();
    bytes[n] = 0x60;
    n += 1; // Empty but valid textual step number.
    bytes[..n].to_vec()
}

#[test]
fn decoder_rejects_oversize_invalid_utf8_indefinite_and_unknown_content() {
    let valid = page(b"light", &[0x65], 3);
    assert!(Response::decode(&valid).is_ok());
    assert_eq!(
        Response::decode(&page(&[b'x'; 513], &[0x79, 2, 1], 3)),
        Err(Error::Bounds)
    );
    assert!(Response::decode(&page(&[0xff], &[0x61], 3)).is_err());
    assert!(Response::decode(&page(b"light", &[0x7f, 0x65], 3)).is_err());
    assert!(Response::decode(&page(b"light", &[0x65], 4)).is_err());
    assert!(
        Response::decode(&page(
            b"light",
            &[0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            3
        ))
        .is_err()
    );
}

#[test]
fn malformed_byte_mutations_never_panic_and_accepted_values_remain_encodable() {
    let mut r = support::response();
    r.request.operation = Operation::Step { index: 0 };
    r.observed.step_count = 1;
    r.body = Body::Step(Some(Step {
        id: [7; 16],
        name: Text::new("灯".repeat(170).as_str()).unwrap(),
        number: Text::new(&"1".repeat(512)).unwrap(),
    }));
    for sample in [r.encode().unwrap(), support::response().encode().unwrap()] {
        let mut candidate = sample.bytes().to_vec();
        for position in 0..candidate.len() {
            for bit in 0..8 {
                candidate[position] ^= 1 << bit;
                if let Ok(decoded) = Response::decode(&candidate) {
                    let encoded = decoded.encode().unwrap();
                    assert!(encoded.bytes().len() <= 1280);
                    assert_eq!(Response::decode(encoded.bytes()).unwrap(), decoded);
                }
                candidate[position] ^= 1 << bit;
            }
        }
    }
}
