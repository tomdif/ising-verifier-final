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
