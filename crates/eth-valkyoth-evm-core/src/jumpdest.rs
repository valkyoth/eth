use crate::{EVM_MAX_BYTECODE_LEN, EvmCoreError, EvmOpcode, bytecode::next_instruction_pc};

const JUMPDEST_MAP_WORD_BITS: usize = usize::BITS as usize;
const JUMPDEST_MAP_WORDS: usize = EVM_MAX_BYTECODE_LEN.div_ceil(JUMPDEST_MAP_WORD_BITS);

pub(crate) struct JumpdestMap {
    words: [usize; JUMPDEST_MAP_WORDS],
    len: usize,
}

impl JumpdestMap {
    pub(crate) fn try_new(bytecode: &[u8]) -> Result<Self, EvmCoreError> {
        if bytecode.len() > EVM_MAX_BYTECODE_LEN {
            return Err(EvmCoreError::BytecodeTooLarge);
        }
        let mut map = Self {
            words: [0usize; JUMPDEST_MAP_WORDS],
            len: bytecode.len(),
        };
        let mut pc = 0usize;
        while pc < bytecode.len() {
            let opcode = EvmOpcode::new(
                *bytecode
                    .get(pc)
                    .ok_or(EvmCoreError::ProgramCounterOverflow)?,
            );
            if opcode.byte() == EvmOpcode::JUMPDEST.byte() {
                map.insert(pc);
            }
            pc = next_instruction_pc(pc, opcode)?;
        }
        Ok(map)
    }

    fn insert(&mut self, target: usize) {
        let word = target / JUMPDEST_MAP_WORD_BITS;
        let bit = target % JUMPDEST_MAP_WORD_BITS;
        if let Some(slot) = self.words.get_mut(word) {
            *slot |= 1usize << bit;
        }
    }

    pub(crate) fn contains(&self, target: usize) -> bool {
        if target >= self.len {
            return false;
        }
        let word = target / JUMPDEST_MAP_WORD_BITS;
        let bit = target % JUMPDEST_MAP_WORD_BITS;
        self.words
            .get(word)
            .is_some_and(|slot| (*slot & (1usize << bit)) != 0)
    }
}
