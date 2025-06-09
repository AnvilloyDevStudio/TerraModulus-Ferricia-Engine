/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use std::sync::atomic::{AtomicUsize, Ordering};

/// Source: https://stackoverflow.com/a/72149089
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct OpaqueId(usize);

impl OpaqueId {
	pub fn new(counter: &'static AtomicUsize) -> Self {
		Self(counter.fetch_add(1, Ordering::Relaxed))
	}

	pub fn id(&self) -> usize {
		self.0
	}
}
