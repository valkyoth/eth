use eth_valkyoth_evm::{
    AccessTracker, CryptoProvider, ExecutionHost, ExecutionRequest, StateJournal, TransactionArena,
};

fn cannot_substitute_host_provenance<'host, 'transaction, J, A, C, R>(
    request: &'host ExecutionRequest<'transaction, J::View>,
    journal: &'host mut J,
    access: &'host mut A,
    crypto: &'host mut C,
    arena: &'host mut R,
) where
    J: StateJournal,
    A: AccessTracker,
    C: CryptoProvider,
    R: TransactionArena,
{
    let _ = ExecutionHost {
        request,
        journal,
        access,
        crypto,
        arena,
        poisoned: false,
        transaction_started: true,
    };
}

fn main() {}
