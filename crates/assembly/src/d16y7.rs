//! Catálogo semilla D16Y7 (S04): definiciones nominales desde el EPC de S01.
//!
//! Solo identidad y puertos; sin dimensiones, sin dinámica, sin curvas.
//! `geometry_ref`/`material_ref` apuntan a trabajo futuro (S07/S31), no a
//! valores.

use crate::graph::EngineGraph;
use crate::types::{DataConfidence, InterfaceSpec, PartDefinition, PortKind};

fn iface(name: &str, kind: PortKind, mates: Option<&str>) -> InterfaceSpec {
    InterfaceSpec {
        name: name.to_owned(),
        kind,
        mates_with_definition: mates.map(str::to_owned),
    }
}

fn def(id: &str, label: &str, ifaces: Vec<InterfaceSpec>, sources: &[&str]) -> PartDefinition {
    PartDefinition {
        id: id.to_owned(),
        label: label.to_owned(),
        geometry_ref: format!("geometry/{id}_v001 (pending S31)"),
        material_ref: format!("material/{id}_unknown (pending S27)"),
        mounting_interfaces: ifaces,
        data_confidence: DataConfidence::CatalogIdentity,
        source_refs: sources.iter().map(|s| (*s).to_owned()).collect(),
    }
}

/// Definiciones del catálogo 1997 DX 5MT (S01 `OEM_CATALOG_1997_DX_5MT.csv`).
#[must_use]
pub fn catalog_definitions() -> Vec<PartDefinition> {
    use PortKind as K;
    vec![
        def(
            "honda.bare_block",
            "Bare block D16Y7 (receptor de todos los conjuntos)",
            vec![
                iface(
                    "main_journals",
                    K::MechanicalMount,
                    Some("honda.crankshaft"),
                ),
                iface(
                    "cylinder_bore_1",
                    K::MechanicalMount,
                    Some("honda.piston.std"),
                ),
                iface(
                    "cylinder_bore_2",
                    K::MechanicalMount,
                    Some("honda.piston.std"),
                ),
                iface(
                    "cylinder_bore_3",
                    K::MechanicalMount,
                    Some("honda.piston.std"),
                ),
                iface(
                    "cylinder_bore_4",
                    K::MechanicalMount,
                    Some("honda.piston.std"),
                ),
                iface("head_deck", K::MechanicalMount, Some("honda.head")),
                iface("oil_gallery", K::OilFlow, Some("honda.oil_pump")),
                iface("coolant_jacket", K::CoolantFlow, None),
            ],
            &["CF15", "CF16", "CF18"],
        ),
        def(
            "honda.crankshaft",
            "Crankshaft (fase 720 continua; inercia en S06/S14)",
            vec![
                iface("main_journal", K::MechanicalMount, Some("honda.bare_block")),
                iface("rod_pin_1", K::TorqueTransmit, Some("honda.rod")),
                iface("rod_pin_2", K::TorqueTransmit, Some("honda.rod")),
                iface("rod_pin_3", K::TorqueTransmit, Some("honda.rod")),
                iface("rod_pin_4", K::TorqueTransmit, Some("honda.rod")),
                iface("pulley_end", K::TorqueTransmit, None),
                iface("flywheel_end", K::TorqueTransmit, None),
            ],
            &["CF16"],
        ),
        def(
            "honda.piston.std",
            "Piston STD 13010-P2E-000 (sobremedidas +0.25/+0.50 como defs futuras)",
            vec![
                iface("skirt", K::Contact, Some("honda.bare_block")),
                iface("pin_bore", K::MechanicalMount, Some("honda.rod")),
                iface("crown", K::Contact, Some("honda.head")),
            ],
            &["CF16"],
        ),
        def(
            "honda.rod",
            "Connecting rod (big/small end clearance por muñequilla en S22)",
            vec![
                iface("big_end", K::TorqueTransmit, Some("honda.crankshaft")),
                iface("small_end", K::MechanicalMount, Some("honda.piston.std")),
            ],
            &["CF16"],
        ),
        def(
            "honda.head",
            "Head assembly 12100-P2A-000",
            vec![
                iface("deck_face", K::MechanicalMount, Some("honda.bare_block")),
                iface("combustion_face", K::Contact, Some("honda.piston.std")),
                iface("oil_passage", K::OilFlow, None),
                iface("coolant_passage", K::CoolantFlow, None),
                iface("intake_port", K::AirFlow, None),
                iface("exhaust_port", K::AirFlow, None),
            ],
            &["CF15"],
        ),
        def(
            "honda.head_gasket",
            "Head gasket 12251-P2J-004 (sellado por regiones en S08/S23)",
            vec![
                iface("fire_ring", K::Contact, None),
                iface("oil_seal", K::OilFlow, None),
                iface("coolant_seal", K::CoolantFlow, None),
            ],
            &["S01:OEM_CATALOG_1997_DX_5MT"],
        ),
        def(
            "honda.head_bolt",
            "Head bolt-washer 90008-PZ1-003 (historial de apriete por instancia)",
            vec![
                iface("thread", K::MechanicalMount, Some("honda.bare_block")),
                iface("bearing_surface", K::MechanicalMount, Some("honda.head")),
            ],
            &["S01:OEM_CATALOG_1997_DX_5MT"],
        ),
        def(
            "honda.oil_pump",
            "Oil pump 15100-P7A-013 (curva Q-dP en S22)",
            vec![iface("outlet", K::OilFlow, Some("honda.bare_block"))],
            &["CF18"],
        ),
        def(
            "honda.distributor",
            "Distributor TD-80U/TD-98U DISPUTED (patrón trigger en S17)",
            vec![
                iface("drive", K::MechanicalMount, None),
                iface("signal_out", K::ElectricalSignal, None),
            ],
            &["CF19"],
        ),
        def(
            "honda.injector",
            "Injector 06164-P2A-000 (Q/deadtime en S13; identidad EPC aquí)",
            vec![
                iface("seat", K::MechanicalMount, None),
                iface("spray", K::AirFlow, None),
                iface("coil", K::ElectricalSignal, None),
            ],
            &["CF17"],
        ),
    ]
}

/// Grafo con catálogo registrado y bloque desnudo `block-001` instalado.
#[must_use]
pub fn bare_block_with_catalog(engine_id: &str) -> EngineGraph {
    let mut g = EngineGraph::empty(engine_id);
    let catalog = catalog_definitions();
    let block_ifaces: Vec<InterfaceSpec> = catalog
        .iter()
        .find(|d| d.id == "honda.bare_block")
        .map(|d| d.mounting_interfaces.clone())
        .unwrap_or_default();
    for d in catalog {
        g.register_definition(d);
    }
    // El bloque desnudo es la única instancia inicial: el resto llega por
    // operaciones (cigüeñal, pistones, culata...), nunca por magia.
    let mut ports = std::collections::BTreeMap::new();
    for iface in &block_ifaces {
        let id = format!("block-001::{}", iface.name);
        ports.insert(
            id.clone(),
            crate::types::Port {
                id,
                instance_serial: "block-001".to_owned(),
                kind: iface.kind,
            },
        );
    }
    g.ports.extend(ports);
    g.instances.insert(
        "block-001".to_owned(),
        crate::types::PartInstance {
            serial: "block-001".to_owned(),
            definition_id: "honda.bare_block".to_owned(),
            cycles: 0,
            history: vec!["bare_block seed (S04)".to_owned()],
        },
    );
    g
}
