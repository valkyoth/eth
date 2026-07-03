#![no_main]

use eth_valkyoth_codec::DecodeLimits;
use eth_valkyoth_verify::{
    MptNode, MptNodeReference, decode_mpt_node, decode_mpt_proof_nodes,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    drive_mpt_node_decode(data, DecodeLimits::TEST_FIXTURE);
    drive_mpt_node_decode(data, DecodeLimits::DEPLOYMENT_STARTING_POINT);
});

fn drive_mpt_node_decode(data: &[u8], limits: DecodeLimits) {
    let Ok(node) = decode_mpt_node(data, limits) else {
        return;
    };
    touch_node(node);

    let nodes = [data];
    let Ok(proof) = decode_mpt_proof_nodes(&nodes, limits) else {
        return;
    };
    let _ = proof.len();
    let _ = proof.encoded_nodes().len();
}

fn touch_node(node: MptNode<'_>) {
    match node {
        MptNode::Branch(branch) => {
            for child in branch.children() {
                let Ok(child) = child else {
                    return;
                };
                touch_reference(child);
            }
            let _ = branch.value();
        }
        MptNode::Extension(extension) => {
            let _ = extension.path.raw().len();
            let _ = extension.path.nibble_count();
            touch_reference(extension.child);
        }
        MptNode::Leaf(leaf) => {
            let _ = leaf.path.raw().len();
            let _ = leaf.path.nibble_count();
            let _ = leaf.value.len();
        }
    }
}

fn touch_reference(reference: MptNodeReference<'_>) {
    match reference {
        MptNodeReference::Empty => {}
        MptNodeReference::Hash(hash) => {
            let _ = hash.to_bytes();
        }
        MptNodeReference::Inline(inline) => {
            let _ = inline.node().map(touch_node);
        }
    }
}
