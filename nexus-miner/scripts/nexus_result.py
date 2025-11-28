#!/usr/bin/env python3
"""
NEXUS Complete Mining Result
Combines docking + ADMET into verifiable result.
"""

import hashlib
import json
from dataclasses import dataclass, asdict
from typing import Optional
from deterministic_admet import compute_deterministic_admet, hash_admet

@dataclass
class NexusResult:
    """Complete mining result - all fields are deterministic"""
    
    # Job identification
    job_id: str
    ligand_id: str
    
    # Docking result (from Vina)
    seed: int
    affinity: str           # Fixed precision: "-7.144"
    docking_hash: str       # SHA256 of pose
    
    # ADMET properties (from RDKit)
    smiles: str
    admet_hash: str         # SHA256 of ADMET canonical string
    
    # Individual ADMET values (for display)
    mw: str
    logp: str
    tpsa: str
    qed: str
    lipinski_pass: str
    bbb_permeable: str
    gi_absorption: str
    
    # Combined verification hash
    result_hash: str        # SHA256(docking_hash || admet_hash)
    
    # Reward calculation
    num_bonds: int
    bond_multiplier: float
    
    @classmethod
    def from_docking(cls, job_id: str, ligand_id: str, seed: int, 
                     affinity: float, docking_hash: str, smiles: str,
                     num_bonds: int):
        """Create result from docking output + SMILES"""
        
        # Compute ADMET
        admet = compute_deterministic_admet(smiles)
        if admet is None:
            raise ValueError(f"Invalid SMILES: {smiles}")
        
        admet_h = hash_admet(admet)
        
        # Combined hash
        combined = hashlib.sha256(
            (docking_hash + admet_h).encode()
        ).hexdigest()
        
        # Bond multiplier (power law)
        multiplier = (num_bonds + 1) ** 1.2
        
        return cls(
            job_id=job_id,
            ligand_id=ligand_id,
            seed=seed,
            affinity=f"{affinity:.3f}",
            docking_hash=docking_hash,
            smiles=smiles,
            admet_hash=admet_h,
            mw=admet["mw"],
            logp=admet["logp"],
            tpsa=admet["tpsa"],
            qed=admet["qed"],
            lipinski_pass=admet["lipinski_pass"],
            bbb_permeable=admet["bbb_permeable"],
            gi_absorption=admet["gi_absorption"],
            result_hash=combined,
            num_bonds=num_bonds,
            bond_multiplier=round(multiplier, 2),
        )
    
    def to_json(self) -> str:
        return json.dumps(asdict(self), indent=2)
    
    def verify(self, other_docking_hash: str, other_smiles: str) -> bool:
        """Verify result against recomputed values"""
        other_admet = compute_deterministic_admet(other_smiles)
        other_admet_hash = hash_admet(other_admet)
        
        expected_combined = hashlib.sha256(
            (other_docking_hash + other_admet_hash).encode()
        ).hexdigest()
        
        return self.result_hash == expected_combined


# Test with MK1 (HIV protease inhibitor from our docking)
if __name__ == "__main__":
    # MK1 SMILES (simplified representation)
    mk1_smiles = "CC(C)(C)NC(=O)C1CC2CCCCC2CN1CC(O)C(CC1=CC=CC=C1)NC(=O)C(CC(N)=O)NC(=O)C1=CC=CC=C1"
    
    # Simulated docking result
    result = NexusResult.from_docking(
        job_id="hiv-protease-1hsg",
        ligand_id="mk1-inhibitor",
        seed=1131634711,
        affinity=-7.543,
        docking_hash="d99af4e54f71bcf3ae75f2b84917be3a50c07549f0d8bb36683d85c72a9709ac",
        smiles=mk1_smiles,
        num_bonds=12
    )
    
    print("=" * 70)
    print("NEXUS MINING RESULT - HIV Protease + MK1")
    print("=" * 70)
    print(f"""
🔬 DOCKING:
   Job: {result.job_id}
   Ligand: {result.ligand_id}
   Affinity: {result.affinity} kcal/mol
   Seed: {result.seed}
   Docking Hash: {result.docking_hash[:32]}...

💊 ADMET:
   MW: {result.mw} | LogP: {result.logp} | TPSA: {result.tpsa}
   QED: {result.qed} | Lipinski: {result.lipinski_pass}
   BBB: {result.bbb_permeable} | GI: {result.gi_absorption}
   ADMET Hash: {result.admet_hash[:32]}...

✅ VERIFICATION:
   Combined Hash: {result.result_hash[:32]}...
   Bonds: {result.num_bonds} → Multiplier: {result.bond_multiplier}x

📄 JSON:
""")
    print(result.to_json())
    
    # Verify
    print("\n🔍 VERIFICATION TEST:")
    is_valid = result.verify(result.docking_hash, mk1_smiles)
    print(f"   Self-verify: {'✅ PASS' if is_valid else '❌ FAIL'}")
