// Copyright 2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Cross-Consensus Message format asset data structures.
//!
//! This encompasses four types for representing assets:
//! - `MultiAsset`: A description of a single asset, either an instance of a non-fungible or some amount of a fungible.
//! - `MultiAssets`: A collection of `MultiAsset`s. These are stored in a `Vec` and sorted with fungibles first.
//! - `Wild`: A single asset wildcard, this can either be "all" assets, or all assets of a specific kind.
//! - `MultiAssetFilter`: A combination of `Wild` and `MultiAssets` designed for efficiently filtering an XCM holding
//!   account.

use super::MultiLocation;
use alloc::{vec, vec::Vec};
use core::{
	cmp::Ordering,
	convert::{TryFrom, TryInto},
	result,
};
use parity_scale_codec::{self as codec, Decode, Encode};
use scale_info::TypeInfo;

/// A general identifier for an instance of a non-fungible asset class.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, TypeInfo)]
pub enum AssetInstance {
	/// Undefined - used if the non-fungible asset class has only one instance.
	Undefined,

	/// A compact index. Technically this could be greater than `u128`, but this implementation supports only
	/// values up to `2**128 - 1`.
	Index(#[codec(compact)] u128),

	/// A 4-byte fixed-length datum.
	Array4([u8; 4]),

	/// An 8-byte fixed-length datum.
	Array8([u8; 8]),

	/// A 16-byte fixed-length datum.
	Array16([u8; 16]),

	/// A 32-byte fixed-length datum.
	Array32([u8; 32]),

	/// An arbitrary piece of data. Use only when necessary.
	Blob(Vec<u8>),
}

impl From<()> for AssetInstance {
	fn from(_: ()) -> Self {
		Self::Undefined
	}
}

impl From<[u8; 4]> for AssetInstance {
	fn from(x: [u8; 4]) -> Self {
		Self::Array4(x)
	}
}

impl From<[u8; 8]> for AssetInstance {
	fn from(x: [u8; 8]) -> Self {
		Self::Array8(x)
	}
}

impl From<[u8; 16]> for AssetInstance {
	fn from(x: [u8; 16]) -> Self {
		Self::Array16(x)
	}
}

impl From<[u8; 32]> for AssetInstance {
	fn from(x: [u8; 32]) -> Self {
		Self::Array32(x)
	}
}

impl From<Vec<u8>> for AssetInstance {
	fn from(x: Vec<u8>) -> Self {
		Self::Blob(x)
	}
}

/// Classification of an asset being concrete or abstract.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum AssetId {
	Concrete(MultiLocation),
	Abstract(Vec<u8>),
}

impl<T: Into<MultiLocation>> From<T> for AssetId {
	fn from(x: T) -> Self {
		Self::Concrete(x.into())
	}
}

impl From<Vec<u8>> for AssetId {
	fn from(x: Vec<u8>) -> Self {
		Self::Abstract(x)
	}
}

impl AssetId {
	/// Prepend a `MultiLocation` to a concrete asset, giving it a new root location.
	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		if let AssetId::Concrete(ref mut l) = self {
			l.prepend_with(prepend.clone()).map_err(|_| ())?;
		}
		Ok(())
	}

	/// Use the value of `self` along with a `fun` fungibility specifier to create the corresponding `MultiAsset` value.
	pub fn into_multiasset(self, fun: Fungibility) -> MultiAsset {
		MultiAsset { fun, id: self }
	}

	/// Use the value of `self` along with a `fun` fungibility specifier to create the corresponding `WildMultiAsset`
	/// wildcard (`AllOf`) value.
	pub fn into_wild(self, fun: WildFungibility) -> WildMultiAsset {
		WildMultiAsset::AllOf { fun, id: self }
	}
}

/// Classification of whether an asset is fungible or not, along with a mandatory amount or instance.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum Fungibility {
	Fungible(#[codec(compact)] u128),
	NonFungible(AssetInstance),
}

impl Fungibility {
	pub fn is_kind(&self, w: WildFungibility) -> bool {
		use Fungibility::*;
		use WildFungibility::{Fungible as WildFungible, NonFungible as WildNonFungible};
		matches!((self, w), (Fungible(_), WildFungible) | (NonFungible(_), WildNonFungible))
	}
}

impl From<u128> for Fungibility {
	fn from(amount: u128) -> Fungibility {
		debug_assert_ne!(amount, 0);
		Fungibility::Fungible(amount)
	}
}

impl<T: Into<AssetInstance>> From<T> for Fungibility {
	fn from(instance: T) -> Fungibility {
		Fungibility::NonFungible(instance.into())
	}
}

#[derive(Clone, Eq, PartialEq, Debug, Encode, Decode, TypeInfo)]
pub struct MultiAsset {
	pub id: AssetId,
	pub fun: Fungibility,
}

impl PartialOrd for MultiAsset {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for MultiAsset {
	fn cmp(&self, other: &Self) -> Ordering {
		match (&self.fun, &other.fun) {
			(Fungibility::Fungible(..), Fungibility::NonFungible(..)) => Ordering::Less,
			(Fungibility::NonFungible(..), Fungibility::Fungible(..)) => Ordering::Greater,
			_ => (&self.id, &self.fun).cmp(&(&other.id, &other.fun)),
		}
	}
}

impl<A: Into<AssetId>, B: Into<Fungibility>> From<(A, B)> for MultiAsset {
	fn from((id, fun): (A, B)) -> MultiAsset {
		MultiAsset { fun: fun.into(), id: id.into() }
	}
}

impl MultiAsset {
	pub fn is_fungible(&self, maybe_id: Option<AssetId>) -> bool {
		use Fungibility::*;
		matches!(self.fun, Fungible(..)) && maybe_id.map_or(true, |i| i == self.id)
	}

	pub fn is_non_fungible(&self, maybe_id: Option<AssetId>) -> bool {
		use Fungibility::*;
		matches!(self.fun, NonFungible(..)) && maybe_id.map_or(true, |i| i == self.id)
	}

	/// Prepend a `MultiLocation` to a concrete asset, giving it a new root location.
	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		self.id.reanchor(prepend)
	}

	/// Prepend a `MultiLocation` to a concrete asset, giving it a new root location.
	pub fn reanchored(mut self, prepend: &MultiLocation) -> Result<Self, ()> {
		self.reanchor(prepend)?;
		Ok(self)
	}

	/// Returns true if `self` is a super-set of the given `inner`.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		use Fungibility::*;
		if self.id == inner.id {
			match (&self.fun, &inner.fun) {
				(Fungible(a), Fungible(i)) if a >= i => return true,
				(NonFungible(a), NonFungible(i)) if a == i => return true,
				_ => (),
			}
		}
		false
	}
}

impl TryFrom<super::super::v0::MultiAsset> for MultiAsset {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<MultiAsset, ()> {
		use super::super::v0::MultiAsset as V0;
		use AssetId::*;
		use Fungibility::*;
		let (id, fun) = match old {
			V0::ConcreteFungible { id, amount } => (Concrete(id.try_into()?), Fungible(amount)),
			V0::ConcreteNonFungible { class, instance } =>
				(Concrete(class.try_into()?), NonFungible(instance)),
			V0::AbstractFungible { id, amount } => (Abstract(id), Fungible(amount)),
			V0::AbstractNonFungible { class, instance } => (Abstract(class), NonFungible(instance)),
			_ => return Err(()),
		};
		Ok(MultiAsset { id, fun })
	}
}

impl TryFrom<super::super::v0::MultiAsset> for Option<MultiAsset> {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<Option<MultiAsset>, ()> {
		match old {
			super::super::v0::MultiAsset::None => return Ok(None),
			x => return Ok(Some(x.try_into()?)),
		}
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAsset {
	type Error = ();
	fn try_from(mut old: Vec<super::super::v0::MultiAsset>) -> result::Result<MultiAsset, ()> {
		if old.len() == 1 {
			old.remove(0).try_into()
		} else {
			Err(())
		}
	}
}

/// A `Vec` of `MultiAsset`s. There may be no duplicate fungible items in here and when decoding, they must be sorted.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, TypeInfo)]
pub struct MultiAssets(Vec<MultiAsset>);

impl Decode for MultiAssets {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
		Self::from_sorted_and_deduplicated(Vec::<MultiAsset>::decode(input)?)
			.map_err(|()| "Out of order".into())
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAssets {
	type Error = ();
	fn try_from(old: Vec<super::super::v0::MultiAsset>) -> result::Result<MultiAssets, ()> {
		let v = old
			.into_iter()
			.map(Option::<MultiAsset>::try_from)
			.filter_map(|x| x.transpose())
			.collect::<result::Result<Vec<MultiAsset>, ()>>()?;
		Ok(v.into())
	}
}

impl From<Vec<MultiAsset>> for MultiAssets {
	fn from(mut assets: Vec<MultiAsset>) -> Self {
		let mut res = Vec::with_capacity(assets.len());
		if !assets.is_empty() {
			assets.sort();
			let mut iter = assets.into_iter();
			if let Some(first) = iter.next() {
				let last = iter.fold(first, |a, b| -> MultiAsset {
					match (a, b) {
						(
							MultiAsset { fun: Fungibility::Fungible(a_amount), id: a_id },
							MultiAsset { fun: Fungibility::Fungible(b_amount), id: b_id },
						) if a_id == b_id => MultiAsset {
							id: a_id,
							fun: Fungibility::Fungible(a_amount.saturating_add(b_amount)),
						},
						(
							MultiAsset { fun: Fungibility::NonFungible(a_instance), id: a_id },
							MultiAsset { fun: Fungibility::NonFungible(b_instance), id: b_id },
						) if a_id == b_id && a_instance == b_instance =>
							MultiAsset { fun: Fungibility::NonFungible(a_instance), id: a_id },
						(to_push, to_remember) => {
							res.push(to_push);
							to_remember
						},
					}
				});
				res.push(last);
			}
		}
		Self(res)
	}
}

impl<T: Into<MultiAsset>> From<T> for MultiAssets {
	fn from(x: T) -> Self {
		Self(vec![x.into()])
	}
}

impl MultiAssets {
	/// A new (empty) value.
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	///
	/// Returns `Ok` if the operation succeeds and `Err` if `r` is out of order or had duplicates. If you can't
	/// guarantee that `r` is sorted and deduplicated, then use `From::<Vec<MultiAsset>>::from` which is infallible.
	pub fn from_sorted_and_deduplicated(r: Vec<MultiAsset>) -> Result<Self, ()> {
		if r.is_empty() {
			return Ok(Self(Vec::new()))
		}
		r.iter().skip(1).try_fold(&r[0], |a, b| -> Result<&MultiAsset, ()> {
			if a.id < b.id || a < b && (a.is_non_fungible(None) || b.is_non_fungible(None)) {
				Ok(b)
			} else {
				Err(())
			}
		})?;
		Ok(Self(r))
	}

	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	///
	/// In release mode, this skips any checks to ensure that `r` is correct, making it a negligible-cost operation.
	/// Generally though you should avoid using it unless you have a strict proof that `r` is valid.
	#[cfg(test)]
	pub fn from_sorted_and_deduplicated_skip_checks(r: Vec<MultiAsset>) -> Self {
		Self::from_sorted_and_deduplicated(r).expect("Invalid input r is not sorted/deduped")
	}
	/// Create a new instance of `MultiAssets` from a `Vec<MultiAsset>` whose contents are sorted and
	/// which contain no duplicates.
	///
	/// In release mode, this skips any checks to ensure that `r` is correct, making it a negligible-cost operation.
	/// Generally though you should avoid using it unless you have a strict proof that `r` is valid.
	///
	/// In test mode, this checks anyway and panics on fail.
	#[cfg(not(test))]
	pub fn from_sorted_and_deduplicated_skip_checks(r: Vec<MultiAsset>) -> Self {
		Self(r)
	}

	/// Add some asset onto the list, saturating. This is quite a laborious operation since it maintains the ordering.
	pub fn push(&mut self, a: MultiAsset) {
		if let Fungibility::Fungible(ref amount) = a.fun {
			for asset in self.0.iter_mut().filter(|x| x.id == a.id) {
				if let Fungibility::Fungible(ref mut balance) = asset.fun {
					*balance = balance.saturating_add(*amount);
					return
				}
			}
		}
		self.0.push(a);
		self.0.sort();
	}

	/// Returns `true` if this definitely represents no asset.
	pub fn is_none(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns true if `self` is a super-set of the given `inner`.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		self.0.iter().any(|i| i.contains(inner))
	}

	/// Consume `self` and return the inner vec.
	pub fn drain(self) -> Vec<MultiAsset> {
		self.0
	}

	/// Return a reference to the inner vec.
	pub fn inner(&self) -> &Vec<MultiAsset> {
		&self.0
	}

	/// Return the number of distinct asset instances contained.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Prepend a `MultiLocation` to any concrete asset items, giving it a new root location.
	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		self.0.iter_mut().try_for_each(|i| i.reanchor(prepend))
	}

	/// Return a reference to an item at a specific index or `None` if it doesn't exist.
	pub fn get(&self, index: usize) -> Option<&MultiAsset> {
		self.0.get(index)
	}
}
/// Classification of whether an asset is fungible or not.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum WildFungibility {
	Fungible,
	NonFungible,
}

/// A wildcard representing a set of assets.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum WildMultiAsset {
	/// All assets in the holding register, up to `usize` individual assets (different instances of non-fungibles could
	/// be separate assets).
	All,
	/// All assets in the holding register of a given fungibility and ID. If operating on non-fungibles, then a limit
	/// is provided for the maximum amount of matching instances.
	AllOf { id: AssetId, fun: WildFungibility },
}

impl TryFrom<super::super::v0::MultiAsset> for WildMultiAsset {
	type Error = ();
	fn try_from(old: super::super::v0::MultiAsset) -> result::Result<WildMultiAsset, ()> {
		use super::super::v0::MultiAsset as V0;
		use AssetId::*;
		use WildFungibility::*;
		let (id, fun) = match old {
			V0::All => return Ok(WildMultiAsset::All),
			V0::AllConcreteFungible { id } => (Concrete(id.try_into()?), Fungible),
			V0::AllConcreteNonFungible { class } => (Concrete(class.try_into()?), NonFungible),
			V0::AllAbstractFungible { id } => (Abstract(id), Fungible),
			V0::AllAbstractNonFungible { class } => (Abstract(class), NonFungible),
			_ => return Err(()),
		};
		Ok(WildMultiAsset::AllOf { id, fun })
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for WildMultiAsset {
	type Error = ();
	fn try_from(mut old: Vec<super::super::v0::MultiAsset>) -> result::Result<WildMultiAsset, ()> {
		if old.len() == 1 {
			old.remove(0).try_into()
		} else {
			Err(())
		}
	}
}

impl WildMultiAsset {
	/// Returns true if `self` is a super-set of the given `inner`.
	///
	/// Typically, any wildcard is never contained in anything else, and a wildcard can contain any other non-wildcard.
	/// For more details, see the implementation and tests.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		use WildMultiAsset::*;
		match self {
			AllOf { fun, id } => inner.fun.is_kind(*fun) && &inner.id == id,
			All => true,
		}
	}

	/// Prepend a `MultiLocation` to any concrete asset components, giving it a new root location.
	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		use WildMultiAsset::*;
		match self {
			AllOf { ref mut id, .. } => id.reanchor(prepend).map_err(|_| ()),
			All => Ok(()),
		}
	}
}

impl<A: Into<AssetId>, B: Into<WildFungibility>> From<(A, B)> for WildMultiAsset {
	fn from((id, fun): (A, B)) -> WildMultiAsset {
		WildMultiAsset::AllOf { fun: fun.into(), id: id.into() }
	}
}

/// `MultiAsset` collection, either `MultiAssets` or a single wildcard.
///
/// Note: Vectors of wildcards whose encoding is supported in XCM v0 are unsupported
/// in this implementation and will result in a decode error.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode, TypeInfo)]
pub enum MultiAssetFilter {
	Definite(MultiAssets),
	Wild(WildMultiAsset),
}

impl<T: Into<WildMultiAsset>> From<T> for MultiAssetFilter {
	fn from(x: T) -> Self {
		Self::Wild(x.into())
	}
}

impl From<MultiAsset> for MultiAssetFilter {
	fn from(x: MultiAsset) -> Self {
		Self::Definite(vec![x].into())
	}
}

impl From<Vec<MultiAsset>> for MultiAssetFilter {
	fn from(x: Vec<MultiAsset>) -> Self {
		Self::Definite(x.into())
	}
}

impl From<MultiAssets> for MultiAssetFilter {
	fn from(x: MultiAssets) -> Self {
		Self::Definite(x)
	}
}

impl MultiAssetFilter {
	/// Returns true if `self` is a super-set of the given `inner`.
	///
	/// Typically, any wildcard is never contained in anything else, and a wildcard can contain any other non-wildcard.
	/// For more details, see the implementation and tests.
	pub fn contains(&self, inner: &MultiAsset) -> bool {
		match self {
			MultiAssetFilter::Definite(ref assets) => assets.contains(inner),
			MultiAssetFilter::Wild(ref wild) => wild.contains(inner),
		}
	}

	/// Prepend a `MultiLocation` to any concrete asset components, giving it a new root location.
	pub fn reanchor(&mut self, prepend: &MultiLocation) -> Result<(), ()> {
		match self {
			MultiAssetFilter::Definite(ref mut assets) => assets.reanchor(prepend),
			MultiAssetFilter::Wild(ref mut wild) => wild.reanchor(prepend),
		}
	}
}

impl TryFrom<Vec<super::super::v0::MultiAsset>> for MultiAssetFilter {
	type Error = ();
	fn try_from(
		mut old: Vec<super::super::v0::MultiAsset>,
	) -> result::Result<MultiAssetFilter, ()> {
		if old.len() == 1 && old[0].is_wildcard() {
			Ok(MultiAssetFilter::Wild(old.remove(0).try_into()?))
		} else {
			Ok(MultiAssetFilter::Definite(old.try_into()?))
		}
	}
}
