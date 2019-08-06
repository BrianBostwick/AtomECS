extern crate specs;
use crate::atom::AtomInfo;
use specs::{
	Component, Entities, HashMapStorage, Join, LazyUpdate, Read, ReadStorage, System, WriteStorage,
};

use crate::constant;

/// A component representing light used for laser cooling.
pub struct CoolingLight {
	/// Polarisation of the laser light, 1. for +, -1. for -,
	pub polarization: f64,

	/// wavelength of the laser light, in SI units of m.
	pub wavelength: f64,
}
impl CoolingLight {
	/// Frequency of the cooling light in units of Hz
	pub fn frequency(&self) -> f64 {
		constant::C / self.wavelength
	}

	/// Wavenumber of the cooling light, in units of 2pi inverse metres.
	pub fn wavenumber(&self) -> f64 {
		2.0 * constant::PI / self.wavelength
	}

	/// Creates a `CoolingLight` component with the specified detuning from the desired atomic species.
	///
	/// # Arguments
	///
	/// `species`: The atomic species to detune from.
	///
	/// `detuning`: Detuning from the transition, specified in MHz. Red-detuned is negative.
	///
	/// `polarization`: Polarization of the cooling beam.
	pub fn for_species(species: AtomInfo, detuning: f64, polarization: f64) -> Self {
		let freq = species.frequency + detuning * 1.0e6;
		CoolingLight {
			wavelength: constant::C / freq,
			polarization: polarization,
		}
	}
}
impl Component for CoolingLight {
	type Storage = HashMapStorage<Self>;
}

/// An index that uniquely identifies this cooling light in the interaction list for each atom.
/// The index value corresponds to the position of each cooling light in the per-atom interaction list array.
pub struct CoolingLightIndex {
	pub index: usize,
}
impl Component for CoolingLightIndex {
	type Storage = HashMapStorage<Self>;
}
impl Default for CoolingLightIndex {
	fn default() -> Self {
		CoolingLightIndex { index: 0 }
	}
}

/// Assigns unique indices to cooling light entities.
///
/// The indices are used to uniquely identify each cooling light when populating the interaction list.
pub struct IndexCoolingLightsSystem;
impl<'a> System<'a> for IndexCoolingLightsSystem {
	type SystemData = (
		ReadStorage<'a, CoolingLight>,
		WriteStorage<'a, CoolingLightIndex>,
	);

	fn run(&mut self, (cooling_light, mut indices): Self::SystemData) {
		let mut iter = 0;
		for (_, mut index) in (&cooling_light, &mut indices).join() {
			index.index = iter;
			iter = iter + 1;
		}
	}
}

/// A system that attaches `CoolingLightIndex` components to entities which have `CoolingLight` but no index.
pub struct AttachIndexToCoolingLightSystem;
impl<'a> System<'a> for AttachIndexToCoolingLightSystem {
	type SystemData = (
		Entities<'a>,
		ReadStorage<'a, CoolingLight>,
		ReadStorage<'a, CoolingLightIndex>,
		Read<'a, LazyUpdate>,
	);

	fn run(&mut self, (ent, cooling_light, cooling_light_index, updater): Self::SystemData) {
		for (ent, _, _) in (&ent, &cooling_light, !&cooling_light_index).join() {
			updater.insert(ent, CoolingLightIndex::default());
		}
	}
}

#[cfg(test)]
pub mod tests {

	use super::*;
	use assert_approx_eq::assert_approx_eq;
	extern crate specs;
	use specs::{Builder, RunNow, World};

	#[test]
	fn test_index_cooling_lights() {
		let mut test_world = World::new();
		test_world.register::<CoolingLightIndex>();
		test_world.register::<CoolingLight>();

		let test_entity_1 = test_world
			.create_entity()
			.with(CoolingLightIndex::default())
			.with(CoolingLight {
				polarization: 1.0,
				wavelength: 780e-9,
			})
			.build();
		let test_entity_2 = test_world
			.create_entity()
			.with(CoolingLightIndex::default())
			.with(CoolingLight {
				polarization: 1.0,
				wavelength: 780e-9,
			})
			.build();

		let mut system = IndexCoolingLightsSystem;
		system.run_now(&test_world.res);
		test_world.maintain();

		let cooling_storage = test_world.read_storage::<CoolingLightIndex>();
		let index_1 = cooling_storage
			.get(test_entity_1)
			.expect("entity not found");
		let index_2 = cooling_storage
			.get(test_entity_2)
			.expect("entity not found");

		assert_ne!(index_1.index, index_2.index);
	}

	#[test]
	fn test_add_index_component_to_cooling_lights() {
		let mut test_world = World::new();
		test_world.register::<CoolingLightIndex>();
		test_world.register::<CoolingLight>();

		let test_entity = test_world
			.create_entity()
			.with(CoolingLight {
				polarization: 1.0,
				wavelength: 780e-9,
			})
			.build();

		let mut system = AttachIndexToCoolingLightSystem;
		system.run_now(&test_world.res);
		test_world.maintain();

		assert_eq!(
			test_world
				.read_storage::<CoolingLightIndex>()
				.get(test_entity)
				.is_none(),
			false
		);
	}

	#[test]
	fn test_for_species() {
		let detuning = 12.0;
		let light = CoolingLight::for_species(AtomInfo::rubidium(), detuning, 1.0);
		assert_approx_eq!(
			light.frequency(),
			AtomInfo::rubidium().frequency + 1.0e6 * detuning
		);
	}
}