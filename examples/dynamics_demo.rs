//! Example: Create ThermalBody and MaterialBody, apply heat, check yield stress.

use kiran::World;
use kiran::dynamics::{MaterialBody, ThermalBody, ThermalPhase};

fn main() {
    let mut world = World::new();

    // Create a steel beam
    let beam = world.spawn();
    world.insert_component(beam, MaterialBody::steel()).unwrap();

    // Create a thermal body (1 kg iron block at room temperature)
    let block = world.spawn();
    world
        .insert_component(
            block,
            ThermalBody::new(293.0, 80.0, 449.0, 1.0).with_phase(ThermalPhase::Solid),
        )
        .unwrap();

    println!("=== Dynamics Demo ===\n");

    // Check yield stress on the steel beam
    let mat = world.get_component::<MaterialBody>(beam).unwrap();
    println!("Steel beam properties:");
    println!("  Young's modulus: {:.0e} Pa", mat.youngs_modulus);
    println!("  Yield strength:  {:.0e} Pa", mat.yield_strength);
    println!("  Density:         {:.0} kg/m3", mat.density);

    let test_stresses = [100e6, 250e6, 300e6];
    for stress in test_stresses {
        let yielded = mat.is_yielded(stress);
        println!("  Stress {:.0e} Pa -> yielded: {yielded}", stress);
    }

    // Apply heat to the thermal body
    println!("\nIron block (1 kg, specific heat 449 J/kg*K):");
    let thermal = world.get_component::<ThermalBody>(block).unwrap();
    println!(
        "  Initial temperature: {:.1} K ({:.1} C)",
        thermal.temperature,
        thermal.temperature - 273.15
    );

    // Apply 4490 J of heat (should raise temp by 10 K)
    let thermal = world.get_component_mut::<ThermalBody>(block).unwrap();
    thermal.apply_heat(4490.0);
    println!(
        "  After +4490 J:       {:.1} K ({:.1} C)",
        thermal.temperature,
        thermal.temperature - 273.15
    );

    // Apply more heat in steps
    for i in 1..=5 {
        let thermal = world.get_component_mut::<ThermalBody>(block).unwrap();
        thermal.apply_heat(10000.0);
        println!(
            "  After +{}0000 J:     {:.1} K ({:.1} C)",
            i,
            thermal.temperature,
            thermal.temperature - 273.15
        );
    }

    let thermal = world.get_component::<ThermalBody>(block).unwrap();
    println!(
        "\nFinal temperature: {:.1} K, phase: {:?}",
        thermal.temperature, thermal.phase
    );
}
