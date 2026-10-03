use crate::{Error, Failure};
use stagemaster_runtime::{Code, Denial};

pub(crate) fn code(f: Failure) -> u8 {
    match f {
        Failure::Storage => 100,
        Failure::Bounds => 101,
        Failure::Runtime(code) => match code {
            Code::Identity => 0,
            Code::Clock => 1,
            Code::Busy => 2,
            Code::Lease => 3,
            Code::Sequence => 4,
            Code::Revision => 5,
            Code::Exhausted => 6,
            Code::Mode => 7,
            Code::Empty => 8,
            Code::Selection => 9,
            Code::NotLoaded => 10,
            Code::Step => 11,
            Code::State => 12,
            Code::Budget => 13,
            Code::Read => 14,
            Code::Integrity => 15,
            Code::Package => 16,
            Code::Allocation => 17,
            Code::Playback => 18,
            Code::Permission(Denial::Missing) => 19,
            Code::Permission(Denial::Expired) => 20,
            Code::Permission(Denial::UncertainTime) => 21,
            Code::Permission(Denial::Restricted) => 22,
        },
    }
}
pub(crate) fn read(code: u8) -> Result<Failure, Error> {
    Ok(match code {
        100 => Failure::Storage,
        101 => Failure::Bounds,
        _ => Failure::Runtime(match code {
            0 => Code::Identity,
            1 => Code::Clock,
            2 => Code::Busy,
            3 => Code::Lease,
            4 => Code::Sequence,
            5 => Code::Revision,
            6 => Code::Exhausted,
            7 => Code::Mode,
            8 => Code::Empty,
            9 => Code::Selection,
            10 => Code::NotLoaded,
            11 => Code::Step,
            12 => Code::State,
            13 => Code::Budget,
            14 => Code::Read,
            15 => Code::Integrity,
            16 => Code::Package,
            17 => Code::Allocation,
            18 => Code::Playback,
            19 => Code::Permission(Denial::Missing),
            20 => Code::Permission(Denial::Expired),
            21 => Code::Permission(Denial::UncertainTime),
            22 => Code::Permission(Denial::Restricted),
            _ => return Err(Error::Format),
        }),
    })
}
