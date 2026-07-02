use eth_valkyoth_derive::RlpDecode;

#[derive(RlpDecode)]
struct Invalid<T> {
    value: T,
}

fn main() {}
