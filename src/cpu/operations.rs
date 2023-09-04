use super::{Cpu, Bus, Operation};

pub struct XXX {}

impl Operation for XXX {
    fn run(&self, _: &mut Cpu, _: &Bus) -> u8 { 0 }
}
