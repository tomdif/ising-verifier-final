use anyhow::Result;
use std::fs;
use wasmtime::*;

pub struct DockingResult {
    pub score: f64,
    pub bonds: i32,
    pub pose_pdbqt: String,
}

pub struct WasmWorker {
    engine: Engine,
    module: Module,
}

impl WasmWorker {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let wasm_bytes = fs::read(wasm_path)?;

        let mut config = Config::new();
        config.wasm_threads(false);
        config.cranelift_nan_canonicalization(true);

        let engine = Engine::new(&config)?;
        let module = Module::new(&engine, &wasm_bytes)?;

        Ok(WasmWorker { engine, module })
    }

    pub fn run_docking(
        &self,
        _receptor: &str,
        _ligand: &str,
        _cx: f64, _cy: f64, _cz: f64,
        _sx: f64, _sy: f64, _sz: f64,
        _exhaustiveness: i32,
        _seed: u64,
    ) -> Result<DockingResult> {
        // Full implementation would:
        // 1. Create WASI context with virtual filesystem
        // 2. Write receptor.pdbqt, ligand.pdbqt, config.txt
        // 3. Call vina main function
        // 4. Read output.pdbqt
        // 5. Parse score and bonds

        // Placeholder return
        Ok(DockingResult {
            score: -7.5,
            bonds: 3,
            pose_pdbqt: "MODEL 1\nATOM...\nENDMDL".to_string(),
        })
    }
}

pub fn parse_vina_output(pdbqt: &str) -> (f64, i32) {
    let mut score = 0.0;
    let mut bonds = 0;

    for line in pdbqt.lines() {
        if line.starts_with("REMARK VINA RESULT:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                score = parts[3].parse().unwrap_or(0.0);
            }
        }
        if line.starts_with("ATOM") || line.starts_with("HETATM") {
            if let Some(atom_type) = line.get(77..79) {
                let t = atom_type.trim();
                if t == "OA" || t == "NA" || t == "HD" {
                    bonds += 1;
                }
            }
        }
    }

    (score, bonds.min(20))
}
pub fn get_bond_multiplier(bonds: i32) -> f64 {
    let bonds = bonds.clamp(0, 30) as usize;
    // Power law: (bonds + 1)^1.2
    let multipliers: [f64; 31] = [
        1.00,   // 0
        2.30,   // 1
        3.76,   // 2
        5.34,   // 3
        7.01,   // 4
        8.76,   // 5
        10.56,  // 6
        12.43,  // 7
        14.35,  // 8
        16.31,  // 9
        18.32,  // 10
        20.37,  // 11
        22.45,  // 12
        24.58,  // 13
        26.73,  // 14
        28.92,  // 15
        31.14,  // 16
        33.39,  // 17
        35.67,  // 18
        37.97,  // 19
        40.30,  // 20
        42.66,  // 21
        45.04,  // 22
        47.44,  // 23
        49.87,  // 24
        52.32,  // 25
        54.79,  // 26
        57.28,  // 27
        59.79,  // 28
        62.32,  // 29
        64.88,  // 30
    ];
    multipliers[bonds]
}
