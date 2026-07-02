use eth_valkyoth_derive::RlpEncode;

#[derive(RlpEncode)]
struct Invalid {
    #[eth_rlp(skip)]
    value: u64,
}

fn main() {}
