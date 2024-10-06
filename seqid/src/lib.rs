use std::fmt::{Display, Formatter as FmtFormatter, Result as FmtResult};
use std::process;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct SequentialObjectId {
    ts: u32,
    machine_hash: u32,
    pid: u16,
    rand_id: u32,
}

impl SequentialObjectId {
    pub fn new() -> SequentialObjectId {
        SequentialObjectId {
            ts: Self::current_ts(),
            machine_hash: Self::machine_hash(),
            pid: Self::pid(),
            rand_id: Self::next_rand_id(),
        }
    }

    pub fn pack(ts: u32, _machine_hash: u32, pid: u16, rand_id: u32) -> [u8; 9] {
        [
            (ts >> 24) as u8,
            (ts >> 16) as u8,
            (ts >> 8) as u8,
            ts as u8,
            (pid >> 8) as u8,
            pid as u8,
            (rand_id >> 16) as u8,
            (rand_id >> 8) as u8,
            rand_id as u8,
        ]
    }

    pub fn unpack(bytes: [u8; 9]) -> (u32, u16, u32) {
        let ts = (bytes[0] as u32) << 24
            | (bytes[1] as u32) << 16
            | (bytes[2] as u32) << 8
            | bytes[3] as u32;
        let pid = (bytes[4] as u16) << 8 | bytes[5] as u16;
        let rand_id = (bytes[6] as u32) << 16 | (bytes[7] as u32) << 8 | bytes[8] as u32;

        (ts, pid, rand_id)
    }

    pub fn machine_hash() -> u32 {
        let host = hostname::get().unwrap_or_default();
        let host = host.into_string().unwrap_or_default();
        let mut hasher = md5::Context::new();
        hasher.consume(host.as_bytes());
        let hash = hasher.compute();
        let bytes = hash.as_ref();
        (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | (bytes[2] as u32)
    }

    pub fn pid() -> u16 {
        process::id() as _
    }

    pub fn next_rand_id() -> u32 {
        lazy_static::lazy_static! {
            static ref NEXT_RAND_ID: AtomicU32 = AtomicU32::new(rand::random());
        }

        NEXT_RAND_ID.fetch_add(1, Ordering::Relaxed) & 0xffffff
    }

    pub fn current_ts() -> u32 {
        chrono::Utc::now().timestamp() as _
    }
}

impl Display for SequentialObjectId {
    fn fmt(&self, f: &mut FmtFormatter<'_>) -> FmtResult {
        let hex = hex::encode(SequentialObjectId::pack(
            self.ts,
            self.machine_hash,
            self.pid,
            self.rand_id,
        ));
        write!(f, "{hex}")
    }
}

// fn main() {
//     println!(
//         "hash: {}, pid: {}, rand: {}, ts: {}",
//         SequentialObjectId::machine_hash(),
//         SequentialObjectId::pid(),
//         SequentialObjectId::next_rand_id(),
//         SequentialObjectId::current_ts()
//     );

//     println!("id: {}", SequentialObjectId::new());
// }

#[cfg(test)]
mod tests {
    use crate::SequentialObjectId;

    #[cfg(test)]
    mod tests {
        use crate::SequentialObjectId;

        #[test]
        fn test_pack() {
            let ts = SequentialObjectId::current_ts();
            let pid = SequentialObjectId::pid();
            let machine_hash = SequentialObjectId::machine_hash();
            let rand_id = SequentialObjectId::next_rand_id();

            let bytes = SequentialObjectId::pack(ts, machine_hash, pid, rand_id);

            assert_eq!(SequentialObjectId::unpack(bytes), (ts, pid, rand_id));
        }
    }
}
