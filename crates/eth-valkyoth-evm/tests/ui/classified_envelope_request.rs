use eth_valkyoth_evm::{ClassifiedEnvelope, ExecutionRequest, StateView};

fn classified_shell_cannot_reach_execution<'a, S: StateView + ?Sized>(
    classified: ClassifiedEnvelope<'a>,
    state: &'a S,
) {
    let _ = ExecutionRequest::new(classified, state);
}

fn main() {}
