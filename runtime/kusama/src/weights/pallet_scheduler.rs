// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.
//! Autogenerated weights for `pallet_scheduler`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-07-01, STEPS: `[50, ]`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("kusama-dev"), DB CACHE: 128

// Executed Command:
// target/release/polkadot
// benchmark
// --chain=kusama-dev
// --steps=50
// --repeat=20
// --pallet=pallet_scheduler
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --header=./file_header.txt
// --output=./runtime/kusama/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_scheduler`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_scheduler::WeightInfo for WeightInfo<T> {
	fn schedule(s: u32) -> Weight {
		(28_202_000 as Weight)
			// Standard Error: 0
			.saturating_add((42_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn cancel(s: u32) -> Weight {
		(27_640_000 as Weight)
			// Standard Error: 14_000
			.saturating_add((5_699_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn schedule_named(s: u32) -> Weight {
		(34_298_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((56_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn cancel_named(s: u32) -> Weight {
		(29_004_000 as Weight)
			// Standard Error: 15_000
			.saturating_add((5_720_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn on_initialize_periodic_named_resolved(_: u32) -> u64 { todo!() }
	fn on_initialize_named_resolved(_: u32) -> u64 { todo!() }
	fn on_initialize_periodic_resolved(_: u32) -> u64 { todo!() }
	fn on_initialize_resolved(_: u32) -> u64 { todo!() }
	fn on_initialize_named_aborted(_: u32) -> u64 { todo!() }
	fn on_initialize_aborted(_: u32) -> u64 { todo!() }
	fn on_initialize_periodic_named(_: u32) -> u64 { todo!() }
	fn on_initialize_periodic(_: u32) -> u64 { todo!() }
	fn on_initialize_named(_: u32) -> u64 { todo!() }
	fn on_initialize(_: u32) -> u64 { todo!() }
}
