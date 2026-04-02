#![no_main]

use libfuzzer_sys::fuzz_target;
use kiran::World;

/// Operations the fuzzer can request on a [`World`].
#[repr(u8)]
enum Op {
    Spawn,
    Despawn,
    InsertU32,
    InsertF64,
    RemoveU32,
    RemoveF64,
    Query,
}

impl Op {
    fn from_byte(b: u8) -> Self {
        match b % 7 {
            0 => Self::Spawn,
            1 => Self::Despawn,
            2 => Self::InsertU32,
            3 => Self::InsertF64,
            4 => Self::RemoveU32,
            5 => Self::RemoveF64,
            _ => Self::Query,
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let mut world = World::new();
    // Track spawned entities so we can target them with later operations.
    let mut entities: Vec<kiran::world::Entity> = Vec::new();

    // Each operation consumes 2 bytes: opcode + parameter.
    let mut iter = data.chunks_exact(2);
    while let Some(chunk) = iter.next() {
        let op = Op::from_byte(chunk[0]);
        let param = chunk[1];

        match op {
            Op::Spawn => {
                entities.push(world.spawn());
            }
            Op::Despawn => {
                if !entities.is_empty() {
                    let idx = (param as usize) % entities.len();
                    let _ = world.despawn(entities[idx]);
                }
            }
            Op::InsertU32 => {
                if !entities.is_empty() {
                    let idx = (param as usize) % entities.len();
                    let _ = world.insert_component(entities[idx], param as u32);
                }
            }
            Op::InsertF64 => {
                if !entities.is_empty() {
                    let idx = (param as usize) % entities.len();
                    let _ = world.insert_component(entities[idx], param as f64);
                }
            }
            Op::RemoveU32 => {
                if !entities.is_empty() {
                    let idx = (param as usize) % entities.len();
                    let _: Option<u32> = world.remove_component(entities[idx]);
                }
            }
            Op::RemoveF64 => {
                if !entities.is_empty() {
                    let idx = (param as usize) % entities.len();
                    let _: Option<f64> = world.remove_component(entities[idx]);
                }
            }
            Op::Query => {
                let _ = world.query::<u32>();
                let _ = world.query::<f64>();
            }
        }
    }
});
