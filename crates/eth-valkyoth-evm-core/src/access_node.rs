use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::{
    EVM_MAX_WARM_ADDRESSES, EVM_MAX_WARM_STORAGE_SLOTS, EvmAccessAttempt, EvmAccessCheckpoint,
    EvmAccessProfile, EvmAccessStatus, EvmAccessTracker, EvmAddress, EvmCoreError, EvmWord,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StorageKey {
    address: EvmAddress,
    key: EvmWord,
}

trait AccessKey: Copy + Ord {
    fn wipe(&mut self);
}

impl AccessKey for EvmAddress {
    fn wipe(&mut self) {
        EvmAddress::wipe(self);
    }
}

impl AccessKey for StorageKey {
    fn wipe(&mut self) {
        EvmAddress::wipe(&mut self.address);
        EvmWord::wipe(&mut self.key);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node<K> {
    key: K,
    left: Option<usize>,
    right: Option<usize>,
    height: usize,
}

impl<K> Node<K> {
    const fn leaf(key: K) -> Self {
        Self {
            key,
            left: None,
            right: None,
            height: 1,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct AvlSet<K: AccessKey> {
    nodes: Vec<Node<K>>,
    root: Option<usize>,
    limit: usize,
}

impl<K: AccessKey> AvlSet<K> {
    fn try_new(limit: usize, hard_limit: usize) -> Result<Self, EvmCoreError> {
        if limit == 0 {
            return Err(EvmCoreError::StateAccessListTooSmall);
        }
        if limit > hard_limit {
            return Err(EvmCoreError::StateAccessCapacityTooLarge);
        }
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(limit)
            .map_err(|_| EvmCoreError::StateAccessAllocationFailed)?;
        Ok(Self {
            nodes,
            root: None,
            limit,
        })
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn allocation_capacity(&self) -> usize {
        self.nodes.capacity()
    }

    fn tree_height(&self) -> Result<usize, EvmCoreError> {
        self.height(self.root)
    }

    fn contains(&self, key: K) -> Result<bool, EvmCoreError> {
        let mut current = self.root;
        let mut visited = 0usize;
        while let Some(index) = current {
            visited = visited
                .checked_add(1)
                .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if visited > self.nodes.len() {
                return Err(EvmCoreError::StateAccessTrackerCorrupt);
            }
            let node = self.node(index)?;
            match key.cmp(&node.key) {
                Ordering::Less => current = node.left,
                Ordering::Equal => return Ok(true),
                Ordering::Greater => current = node.right,
            }
        }
        Ok(false)
    }

    fn can_insert(&self, key: K) -> Result<bool, EvmCoreError> {
        if self.contains(key)? {
            return Ok(false);
        }
        if self.nodes.len() >= self.limit {
            return Err(EvmCoreError::StateAccessListFull);
        }
        Ok(true)
    }

    fn insert_known_absent(&mut self, key: K) -> Result<(), EvmCoreError> {
        if self.nodes.len() >= self.limit {
            return Err(EvmCoreError::StateAccessListFull);
        }
        let index = self.nodes.len();
        self.nodes.push(Node::leaf(key));
        self.root = Some(match self.root {
            Some(root) => self.insert_index(root, index)?,
            None => index,
        });
        Ok(())
    }

    fn clear(&mut self) {
        for node in &mut self.nodes {
            node.key.wipe();
        }
        self.nodes.clear();
        self.root = None;
    }

    fn restore_len(&mut self, len: usize) -> Result<(), EvmCoreError> {
        if len > self.nodes.len() {
            return Err(EvmCoreError::StateAccessTrackerCorrupt);
        }
        for node in self.nodes.iter_mut().skip(len) {
            node.key.wipe();
        }
        self.nodes.truncate(len);
        self.root = None;
        for node in &mut self.nodes {
            node.left = None;
            node.right = None;
            node.height = 1;
        }
        for index in 0..self.nodes.len() {
            self.root = Some(match self.root {
                Some(root) => self.insert_index(root, index)?,
                None => index,
            });
        }
        Ok(())
    }

    fn insert_index(&mut self, root: usize, index: usize) -> Result<usize, EvmCoreError> {
        let key = self.node(index)?.key;
        let ordering = key.cmp(&self.node(root)?.key);
        match ordering {
            Ordering::Less => {
                let child = self.node(root)?.left;
                let inserted = match child {
                    Some(child) => self.insert_index(child, index)?,
                    None => index,
                };
                self.node_mut(root)?.left = Some(inserted);
            }
            Ordering::Greater => {
                let child = self.node(root)?.right;
                let inserted = match child {
                    Some(child) => self.insert_index(child, index)?,
                    None => index,
                };
                self.node_mut(root)?.right = Some(inserted);
            }
            Ordering::Equal => return Err(EvmCoreError::StateAccessTrackerCorrupt),
        }
        self.update_height(root)?;
        self.rebalance(root)
    }

    fn rebalance(&mut self, root: usize) -> Result<usize, EvmCoreError> {
        let left = self.node(root)?.left;
        let right = self.node(root)?.right;
        let left_height = self.height(left)?;
        let right_height = self.height(right)?;
        if left_height > right_height.saturating_add(1) {
            let left_index = left.ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if self.height(self.node(left_index)?.right)?
                > self.height(self.node(left_index)?.left)?
            {
                let rotated = self.rotate_left(left_index)?;
                self.node_mut(root)?.left = Some(rotated);
            }
            return self.rotate_right(root);
        }
        if right_height > left_height.saturating_add(1) {
            let right_index = right.ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
            if self.height(self.node(right_index)?.left)?
                > self.height(self.node(right_index)?.right)?
            {
                let rotated = self.rotate_right(right_index)?;
                self.node_mut(root)?.right = Some(rotated);
            }
            return self.rotate_left(root);
        }
        Ok(root)
    }

    fn rotate_left(&mut self, root: usize) -> Result<usize, EvmCoreError> {
        let pivot = self
            .node(root)?
            .right
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let inner = self.node(pivot)?.left;
        self.node_mut(root)?.right = inner;
        self.node_mut(pivot)?.left = Some(root);
        self.update_height(root)?;
        self.update_height(pivot)?;
        Ok(pivot)
    }

    fn rotate_right(&mut self, root: usize) -> Result<usize, EvmCoreError> {
        let pivot = self
            .node(root)?
            .left
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let inner = self.node(pivot)?.right;
        self.node_mut(root)?.left = inner;
        self.node_mut(pivot)?.right = Some(root);
        self.update_height(root)?;
        self.update_height(pivot)?;
        Ok(pivot)
    }

    fn update_height(&mut self, index: usize) -> Result<(), EvmCoreError> {
        let left = self.node(index)?.left;
        let right = self.node(index)?.right;
        let height = self
            .height(left)?
            .max(self.height(right)?)
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        self.node_mut(index)?.height = height;
        Ok(())
    }

    fn height(&self, index: Option<usize>) -> Result<usize, EvmCoreError> {
        match index {
            Some(index) => Ok(self.node(index)?.height),
            None => Ok(0),
        }
    }

    fn node(&self, index: usize) -> Result<&Node<K>, EvmCoreError> {
        self.nodes
            .get(index)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)
    }

    fn node_mut(&mut self, index: usize) -> Result<&mut Node<K>, EvmCoreError> {
        self.nodes
            .get_mut(index)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)
    }
}

impl<K: AccessKey> Drop for AvlSet<K> {
    fn drop(&mut self) {
        self.clear();
    }
}

/// Pre-reserved AVL-backed tracker for node-scale execution.
///
/// Construction performs all allocation. Warm-address and warm-storage
/// operations allocate nothing afterward and take worst-case `O(log n)`
/// comparisons and rotations. `reset_transaction` is `O(n)` and retains the
/// original bounded allocation. Rollback rebuilds the checkpointed AVL indexes
/// in `O(n log n)` without allocating.
#[derive(Debug, Eq, PartialEq)]
pub struct EvmNodeAccessTracker {
    addresses: AvlSet<EvmAddress>,
    storage: AvlSet<StorageKey>,
    attempt: Option<EvmAccessCheckpoint>,
    generation: u64,
    checkpoint_depth: usize,
}

impl EvmNodeAccessTracker {
    /// Creates an empty tracker and reserves both exact logical capacities.
    pub fn try_new(address_capacity: usize, storage_capacity: usize) -> Result<Self, EvmCoreError> {
        Ok(Self {
            addresses: AvlSet::try_new(address_capacity, EVM_MAX_WARM_ADDRESSES)?,
            storage: AvlSet::try_new(storage_capacity, EVM_MAX_WARM_STORAGE_SLOTS)?,
            attempt: None,
            generation: 0,
            checkpoint_depth: 0,
        })
    }

    /// Returns the number of warmed addresses.
    #[must_use]
    pub fn address_len(&self) -> usize {
        self.addresses.len()
    }

    /// Returns the number of warmed storage slots.
    #[must_use]
    pub fn storage_len(&self) -> usize {
        self.storage.len()
    }

    /// Returns allocator capacities retained across transaction resets.
    #[must_use]
    pub fn allocation_capacities(&self) -> (usize, usize) {
        (
            self.addresses.allocation_capacity(),
            self.storage.allocation_capacity(),
        )
    }

    /// Returns address and storage AVL heights for complexity evidence.
    pub fn tree_heights(&self) -> Result<(usize, usize), EvmCoreError> {
        Ok((self.addresses.tree_height()?, self.storage.tree_height()?))
    }

    /// Destructively clears all transaction warmth while retaining allocation.
    pub fn reset_transaction(&mut self) -> Result<(), EvmCoreError> {
        <Self as EvmAccessTracker>::reset_transaction(self)
    }

    /// Marks an address warm.
    pub fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError> {
        <Self as EvmAccessTracker>::warm_address(self, address)
    }

    /// Marks an address/storage pair warm.
    pub fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError> {
        <Self as EvmAccessTracker>::warm_storage(self, address, key)
    }

    fn validate_checkpoint(&self, checkpoint: &EvmAccessCheckpoint) -> Result<(), EvmCoreError> {
        if checkpoint.generation != self.generation
            || checkpoint.depth == 0
            || checkpoint.depth != self.checkpoint_depth
        {
            return Err(EvmCoreError::StateAccessAttemptMissing);
        }
        Ok(())
    }
}

impl EvmAccessTracker for EvmNodeAccessTracker {
    type Checkpoint = EvmAccessCheckpoint;

    fn profile(&self) -> EvmAccessProfile {
        EvmAccessProfile::NodeLogarithmic
    }

    fn address_len(&self) -> usize {
        self.addresses.len()
    }

    fn storage_len(&self) -> usize {
        self.storage.len()
    }

    fn reset_transaction(&mut self) -> Result<(), EvmCoreError> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        self.addresses.clear();
        self.storage.clear();
        self.attempt = None;
        self.checkpoint_depth = 0;
        self.generation = generation;
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, EvmCoreError> {
        let depth = self
            .checkpoint_depth
            .checked_add(1)
            .ok_or(EvmCoreError::StateAccessTrackerCorrupt)?;
        let checkpoint = EvmAccessCheckpoint {
            address_len: self.addresses.len(),
            storage_len: self.storage.len(),
            generation: self.generation,
            depth,
        };
        self.checkpoint_depth = depth;
        Ok(checkpoint)
    }

    fn commit(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
        Ok(())
    }

    fn revert(&mut self, checkpoint: Self::Checkpoint) -> Result<(), EvmCoreError> {
        self.validate_checkpoint(&checkpoint)?;
        self.addresses.restore_len(checkpoint.address_len)?;
        self.storage.restore_len(checkpoint.storage_len)?;
        self.checkpoint_depth = self.checkpoint_depth.saturating_sub(1);
        Ok(())
    }

    fn begin_attempt(&mut self) -> Result<(), EvmCoreError> {
        if self.attempt.is_some() {
            return Err(EvmCoreError::StateAccessAttemptAlreadyActive);
        }
        self.attempt = Some(self.checkpoint()?);
        Ok(())
    }

    fn finish_attempt(&mut self, outcome: EvmAccessAttempt) -> Result<(), EvmCoreError> {
        let checkpoint = self
            .attempt
            .take()
            .ok_or(EvmCoreError::StateAccessAttemptMissing)?;
        match outcome {
            EvmAccessAttempt::Commit => self.commit(checkpoint),
            EvmAccessAttempt::Rollback => self.revert(checkpoint),
        }
    }

    fn warm_address(&mut self, address: EvmAddress) -> Result<EvmAccessStatus, EvmCoreError> {
        if !self.addresses.can_insert(address)? {
            return Ok(EvmAccessStatus::Warm);
        }
        self.addresses.insert_known_absent(address)?;
        Ok(EvmAccessStatus::Cold)
    }

    fn warm_storage(
        &mut self,
        address: EvmAddress,
        key: EvmWord,
    ) -> Result<EvmAccessStatus, EvmCoreError> {
        let storage = StorageKey { address, key };
        if self.storage.contains(storage)? {
            return Ok(EvmAccessStatus::Warm);
        }
        let insert_address = self.addresses.can_insert(address)?;
        let _ = self.storage.can_insert(storage)?;
        if insert_address {
            self.addresses.insert_known_absent(address)?;
        }
        self.storage.insert_known_absent(storage)?;
        Ok(EvmAccessStatus::Cold)
    }
}

#[cfg(test)]
#[path = "access_node_wipe_tests.rs"]
mod wipe_tests;
