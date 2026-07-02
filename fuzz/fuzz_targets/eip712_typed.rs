#![no_main]

use eth_valkyoth_hash::Keccak256;
use eth_valkyoth_primitives::{Address, B256};
use eth_valkyoth_verify::{
    Eip712Field, Eip712StructType, Eip712Value, Eip712ValueKind, eip712_hash_struct,
    encode_eip712_type,
};
use libfuzzer_sys::fuzz_target;

const MAX_TYPES: usize = 4;
const MAX_FIELDS: usize = 4;
const SCRATCH_BYTES: usize = 512;
const OUTPUT_BYTES: usize = 512;

static ARRAY_VALUES: [Eip712ValueKind<'static>; 2] =
    [Eip712ValueKind::Uint64(1), Eip712ValueKind::Uint64(2)];
static BYTE_VALUES: [u8; 32] = [0x42_u8; 32];

const STRUCT_NAMES: [&str; 8] = [
    "Root", "Person", "Mail", "Asset", "address", "uint256", "bytes32", "Node",
];
const FIELD_NAMES: [&str; 6] = ["from", "to", "value", "nonce", "tag", "items"];
const FIELD_TYPES: [&str; 14] = [
    "address",
    "bool",
    "bytes",
    "string",
    "uint8",
    "uint64",
    "uint256",
    "int256",
    "bytes1",
    "bytes32",
    "Person",
    "Asset",
    "uint256[]",
    "Person[]",
];

fuzz_target!(|data: &[u8]| {
    let mut reader = Reader::new(data);
    let type_count = reader.bounded_len(MAX_TYPES);
    let mut field_sets = Vec::<Vec<Eip712Field<'static>>>::new();
    for _ in 0..type_count {
        let field_count = reader.bounded_len(MAX_FIELDS);
        let mut fields = Vec::new();
        for _ in 0..field_count {
            fields.push(Eip712Field {
                name: reader.pick(&FIELD_NAMES),
                type_name: reader.pick(&FIELD_TYPES),
            });
        }
        field_sets.push(fields);
    }

    let mut types = Vec::new();
    for fields in &field_sets {
        types.push(Eip712StructType {
            name: reader.pick(&STRUCT_NAMES),
            fields,
        });
    }

    let primary_type = reader.pick(&STRUCT_NAMES);
    let mut scratch = [0_u8; SCRATCH_BYTES];
    let _ = encode_eip712_type(&types, primary_type, &mut scratch);

    let Some(primary) = types.iter().find(|ty| ty.name == primary_type).copied() else {
        return;
    };
    let mut values = Vec::<Eip712Value<'_>>::new();
    for field in primary.fields {
        let value = fuzz_value_for_type(field.type_name, &mut reader);
        values.push(Eip712Value {
            name: field.name,
            value,
        });
    }
    let mut hash_scratch = [0_u8; OUTPUT_BYTES];
    let _ = eip712_hash_struct::<FuzzKeccak>(&types, primary_type, &values, &mut hash_scratch);
});

fn fuzz_value_for_type(
    type_name: &str,
    reader: &mut Reader<'_>,
) -> Eip712ValueKind<'static> {
    if type_name.ends_with("[]") {
        return Eip712ValueKind::Array(&ARRAY_VALUES);
    }
    match type_name {
        "address" => Eip712ValueKind::Address(Address::from_bytes(reader.address_bytes())),
        "bool" => Eip712ValueKind::Bool(reader.next_bool()),
        "bytes" => Eip712ValueKind::Bytes(&BYTE_VALUES),
        "string" => Eip712ValueKind::String(reader.small_str()),
        "int256" => Eip712ValueKind::Int256(reader.word_bytes()),
        name if name.starts_with("bytes") => Eip712ValueKind::FixedBytes(&BYTE_VALUES),
        name if name.starts_with("uint") => Eip712ValueKind::Uint256(reader.word_bytes()),
        _ => Eip712ValueKind::Struct(&[]),
    }
}

struct Reader<'a> {
    data: &'a [u8],
    cursor: usize,
}

impl<'a> Reader<'a> {
    const fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    fn bounded_len(&mut self, max: usize) -> usize {
        let byte = usize::from(self.next_byte());
        byte.checked_rem(max.saturating_add(1)).unwrap_or_default()
    }

    fn pick(&mut self, values: &'static [&'static str]) -> &'static str {
        let index = usize::from(self.next_byte())
            .checked_rem(values.len())
            .unwrap_or_default();
        values.get(index).copied().unwrap_or_default()
    }

    fn next_bool(&mut self) -> bool {
        self.next_byte() & u8::from(true) == u8::from(true)
    }

    fn address_bytes(&mut self) -> [u8; 20] {
        let mut bytes = [0_u8; 20];
        self.fill(&mut bytes);
        bytes
    }

    fn word_bytes(&mut self) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        self.fill(&mut bytes);
        bytes
    }

    fn small_str(&mut self) -> &'static str {
        if self.next_bool() { "alpha" } else { "beta" }
    }

    fn fill(&mut self, output: &mut [u8]) {
        for byte in output {
            *byte = self.next_byte();
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.data.get(self.cursor).copied().unwrap_or_default();
        self.cursor = self.cursor.saturating_add(1);
        byte
    }
}

#[derive(Default)]
struct FuzzKeccak {
    digest: [u8; 32],
    cursor: usize,
}

impl Keccak256 for FuzzKeccak {
    fn update(&mut self, input: &[u8]) {
        for byte in input {
            let Some(slot) = self.digest.get_mut(self.cursor) else {
                self.cursor = 0;
                continue;
            };
            *slot ^= *byte;
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    fn finalize(self) -> B256 {
        B256::from_bytes(self.digest)
    }
}
