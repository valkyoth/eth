use alloc::vec::Vec;
use core::fmt::Debug;

use crate::EvmCoreError;

pub(crate) trait AccessKey: Copy + Debug + Ord {
    const BITS: usize;

    fn bit(&self, index: usize) -> bool;
    fn wipe(&mut self);

    fn first_differing_bit(&self, other: &Self) -> Option<usize> {
        (0..Self::BITS).find(|index| self.bit(*index) != other.bit(*index))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RadixNode<K> {
    Leaf(K),
    Branch { bit: usize, zero: usize, one: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParentUndo {
    index: usize,
    one: bool,
    child: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InsertUndo {
    leaf_len: usize,
    root: Option<usize>,
    parent: Option<ParentUndo>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SetCheckpoint {
    nodes_len: usize,
    leaf_len: usize,
    undo_len: usize,
    root: Option<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RadixSet<K: AccessKey> {
    nodes: Vec<RadixNode<K>>,
    undo: Vec<InsertUndo>,
    root: Option<usize>,
    leaf_len: usize,
    limit: usize,
}

impl<K: AccessKey> RadixSet<K> {
    pub(crate) fn try_new(limit: usize, hard_limit: usize) -> Result<Self, EvmCoreError> {
        if limit == 0 {
            return Err(EvmCoreError::StateAccessListTooSmall);
        }
        if limit > hard_limit {
            return Err(EvmCoreError::StateAccessCapacityTooLarge);
        }
        let node_capacity = limit
            .checked_mul(2)
            .and_then(|value| value.checked_sub(1))
            .ok_or(EvmCoreError::StateAccessCapacityTooLarge)?;
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(node_capacity)
            .map_err(|_| EvmCoreError::StateAccessAllocationFailed)?;
        let mut undo = Vec::new();
        undo.try_reserve_exact(limit)
            .map_err(|_| EvmCoreError::StateAccessAllocationFailed)?;
        Ok(Self {
            nodes,
            undo,
            root: None,
            leaf_len: 0,
            limit,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.leaf_len
    }

    pub(crate) fn allocation_capacity(&self) -> usize {
        self.nodes.capacity().saturating_add(self.undo.capacity())
    }

    pub(crate) fn checkpoint(&self) -> SetCheckpoint {
        SetCheckpoint {
            nodes_len: self.nodes.len(),
            leaf_len: self.leaf_len,
            undo_len: self.undo.len(),
            root: self.root,
        }
    }

    pub(crate) fn contains(&self, key: K) -> Result<bool, EvmCoreError> {
        let Some(leaf) = self.find_leaf(key)? else {
            return Ok(false);
        };
        match self.node(leaf)? {
            RadixNode::Leaf(existing) => Ok(*existing == key),
            RadixNode::Branch { .. } => Err(EvmCoreError::StateAccessTrackerCorrupt),
        }
    }

    pub(crate) fn can_insert(&self, key: K) -> Result<bool, EvmCoreError> {
        if self.contains(key)? {
            return Ok(false);
        }
        let required_nodes = if self.root.is_some() { 2 } else { 1 };
        if self.leaf_len >= self.limit
            || self.nodes.len().saturating_add(required_nodes) > self.nodes.capacity()
            || self.undo.len() >= self.undo.capacity()
        {
            return Err(EvmCoreError::StateAccessListFull);
        }
        Ok(true)
    }

    pub(crate) fn insert_known_absent(&mut self, key: K) -> Result<(), EvmCoreError> {
        let old_root = self.root;
        let old_nodes_len = self.nodes.len();
        let old_leaf_len = self.leaf_len;
        let next_leaf_len = self
            .leaf_len
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        let Some(root) = old_root else {
            self.record_insert(old_leaf_len, old_root, None, key);
            self.root = Some(old_nodes_len);
            self.leaf_len = next_leaf_len;
            return Ok(());
        };

        let leaf = self
            .find_leaf(key)?
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let existing = match self.node(leaf)? {
            RadixNode::Leaf(existing) => *existing,
            RadixNode::Branch { .. } => return Err(EvmCoreError::StateAccessTrackerCorrupt),
        };
        let differing = key
            .first_differing_bit(&existing)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let (current, parent) = self.insertion_point(root, key, differing)?;
        let leaf_index = old_nodes_len;
        let branch_index = leaf_index
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessListFull)?;
        let direction = key.bit(differing);
        let (zero, one) = if direction {
            (current, leaf_index)
        } else {
            (leaf_index, current)
        };
        self.undo.push(InsertUndo {
            leaf_len: old_leaf_len,
            root: old_root,
            parent,
        });
        self.nodes.push(RadixNode::Leaf(key));
        self.nodes.push(RadixNode::Branch {
            bit: differing,
            zero,
            one,
        });
        match parent {
            Some(parent) => self.set_child(parent.index, parent.one, branch_index)?,
            None => self.root = Some(branch_index),
        }
        self.leaf_len = next_leaf_len;
        Ok(())
    }

    pub(crate) fn restore(&mut self, checkpoint: SetCheckpoint) -> Result<(), EvmCoreError> {
        if checkpoint.nodes_len > self.nodes.len()
            || checkpoint.leaf_len > self.leaf_len
            || checkpoint.undo_len > self.undo.len()
        {
            return Err(EvmCoreError::StateAccessTrackerCorrupt);
        }
        while self.undo.len() > checkpoint.undo_len {
            let undo = self
                .undo
                .pop()
                .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if let Some(parent) = undo.parent {
                self.set_child(parent.index, parent.one, parent.child)?;
            }
            self.root = undo.root;
            self.leaf_len = undo.leaf_len;
        }
        self.wipe_from(checkpoint.nodes_len);
        self.nodes.truncate(checkpoint.nodes_len);
        self.root = checkpoint.root;
        self.leaf_len = checkpoint.leaf_len;
        Ok(())
    }

    pub(crate) fn clear_undo(&mut self) {
        self.undo.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.wipe_from(0);
        self.nodes.clear();
        self.undo.clear();
        self.root = None;
        self.leaf_len = 0;
    }

    pub(crate) fn max_depth(&self) -> Result<usize, EvmCoreError> {
        let mut maximum = 0usize;
        for node in &self.nodes {
            if let RadixNode::Leaf(key) = node {
                maximum = maximum.max(self.lookup_depth(*key)?);
            }
        }
        Ok(maximum)
    }

    fn record_insert(
        &mut self,
        leaf_len: usize,
        root: Option<usize>,
        parent: Option<ParentUndo>,
        key: K,
    ) {
        self.undo.push(InsertUndo {
            leaf_len,
            root,
            parent,
        });
        self.nodes.push(RadixNode::Leaf(key));
    }

    fn insertion_point(
        &self,
        root: usize,
        key: K,
        differing: usize,
    ) -> Result<(usize, Option<ParentUndo>), EvmCoreError> {
        let mut current = root;
        let mut parent = None;
        loop {
            match self.node(current)? {
                RadixNode::Leaf(_) => return Ok((current, parent)),
                RadixNode::Branch { bit, .. } if *bit >= differing => {
                    return Ok((current, parent));
                }
                RadixNode::Branch { bit, zero, one } => {
                    let direction = key.bit(*bit);
                    parent = Some(ParentUndo {
                        index: current,
                        one: direction,
                        child: if direction { *one } else { *zero },
                    });
                    current = if direction { *one } else { *zero };
                }
            }
        }
    }

    fn lookup_depth(&self, key: K) -> Result<usize, EvmCoreError> {
        let mut current = self.root;
        let mut depth = 0usize;
        while let Some(index) = current {
            depth = depth
                .checked_add(1)
                .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if depth > K::BITS.saturating_add(1) {
                return Err(EvmCoreError::StateAccessTrackerCorrupt);
            }
            match self.node(index)? {
                RadixNode::Leaf(_) => return Ok(depth),
                RadixNode::Branch { bit, zero, one } => {
                    current = Some(if key.bit(*bit) { *one } else { *zero });
                }
            }
        }
        Ok(depth)
    }

    fn find_leaf(&self, key: K) -> Result<Option<usize>, EvmCoreError> {
        let mut current = self.root;
        let mut depth = 0usize;
        while let Some(index) = current {
            depth = depth
                .checked_add(1)
                .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if depth > K::BITS.saturating_add(1) {
                return Err(EvmCoreError::StateAccessTrackerCorrupt);
            }
            match self.node(index)? {
                RadixNode::Leaf(_) => return Ok(Some(index)),
                RadixNode::Branch { bit, zero, one } => {
                    current = Some(if key.bit(*bit) { *one } else { *zero });
                }
            }
        }
        Ok(None)
    }

    fn set_child(&mut self, index: usize, one: bool, child: usize) -> Result<(), EvmCoreError> {
        match self.node_mut(index)? {
            RadixNode::Branch {
                zero,
                one: one_child,
                ..
            } => {
                if one {
                    *one_child = child;
                } else {
                    *zero = child;
                }
                Ok(())
            }
            RadixNode::Leaf(_) => Err(EvmCoreError::StateAccessTrackerCorrupt),
        }
    }

    fn wipe_from(&mut self, index: usize) {
        for node in self.nodes.iter_mut().skip(index) {
            if let RadixNode::Leaf(key) = node {
                key.wipe();
            }
        }
    }

    fn node(&self, index: usize) -> Result<&RadixNode<K>, EvmCoreError> {
        self.nodes
            .get(index)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)
    }

    fn node_mut(&mut self, index: usize) -> Result<&mut RadixNode<K>, EvmCoreError> {
        self.nodes
            .get_mut(index)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)
    }
}

impl<K: AccessKey> Drop for RadixSet<K> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
#[path = "access_radix_tests.rs"]
mod tests;
