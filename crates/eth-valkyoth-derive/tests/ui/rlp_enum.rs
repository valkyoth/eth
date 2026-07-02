use eth_valkyoth_derive::RlpEncode;

#[derive(RlpEncode)]
enum Invalid {
    A(u64),
}

fn main() {}
