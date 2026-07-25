use eth_valkyoth_codec::{DecodeSession, DecodeSessionCharges};

use super::{StateProofVerificationError, proof_resource_error};

pub(super) struct PlannedStateValue<T> {
    pub(super) charges: DecodeSessionCharges,
    pub(super) value: T,
}

pub(super) fn decode_state_value<T>(
    session: &mut DecodeSession,
    operation: impl FnOnce(&mut DecodeSession) -> Result<T, StateProofVerificationError>,
) -> Result<PlannedStateValue<T>, StateProofVerificationError> {
    let mut future = DecodeSession::new(session.policy()).map_err(proof_resource_error)?;
    let result = operation(&mut future);
    let charges = future.charges();
    match result {
        Ok(value) => Ok(PlannedStateValue { charges, value }),
        Err(error) => {
            session
                .account_charges(charges)
                .map_err(proof_resource_error)?;
            Err(error)
        }
    }
}

pub(super) fn check_state_capacity(
    session: &DecodeSession,
    proof: DecodeSessionCharges,
    state_decode: DecodeSessionCharges,
) -> Result<(), StateProofVerificationError> {
    let mut combined = DecodeSession::new(session.policy()).map_err(proof_resource_error)?;
    combined
        .account_charges(proof)
        .and_then(|()| combined.account_charges(state_decode))
        .map_err(proof_resource_error)?;
    session
        .check_remaining_capacity(combined.charges())
        .map_err(proof_resource_error)?;
    Ok(())
}

pub(super) fn commit_state_decode(
    session: &mut DecodeSession,
    charges: DecodeSessionCharges,
) -> Result<(), StateProofVerificationError> {
    session
        .account_charges(charges)
        .map_err(proof_resource_error)?;
    Ok(())
}
