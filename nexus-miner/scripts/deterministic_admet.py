#!/usr/bin/env python3
"""
Deterministic ADMET Calculator
All outputs are canonical strings suitable for hashing.
"""

import hashlib
import json
from rdkit import Chem
from rdkit.Chem import Descriptors, Lipinski, QED, rdMolDescriptors

def compute_deterministic_admet(smiles: str) -> dict:
    """
    Compute ADMET properties with deterministic string outputs.
    All floats are formatted to fixed precision.
    """
    mol = Chem.MolFromSmiles(smiles)
    if mol is None:
        return None
    
    # Compute all properties
    result = {
        # Use fixed decimal places for determinism
        "mw": f"{Descriptors.MolWt(mol):.3f}",
        "logp": f"{Descriptors.MolLogP(mol):.3f}", 
        "tpsa": f"{Descriptors.TPSA(mol):.2f}",
        "hbd": str(Lipinski.NumHDonors(mol)),
        "hba": str(Lipinski.NumHAcceptors(mol)),
        "rotatable": str(Lipinski.NumRotatableBonds(mol)),
        "rings": str(rdMolDescriptors.CalcNumRings(mol)),
        "aromatic_rings": str(rdMolDescriptors.CalcNumAromaticRings(mol)),
        "heavy_atoms": str(mol.GetNumHeavyAtoms()),
        "qed": f"{QED.qed(mol):.4f}",
        "fraction_csp3": f"{rdMolDescriptors.CalcFractionCSP3(mol):.4f}",
    }
    
    # Compute derived boolean properties (deterministic)
    mw = float(result["mw"])
    logp = float(result["logp"])
    tpsa = float(result["tpsa"])
    hbd = int(result["hbd"])
    hba = int(result["hba"])
    
    result["lipinski_pass"] = str(
        mw <= 500 and logp <= 5 and hbd <= 5 and hba <= 10
    ).lower()
    
    result["veber_pass"] = str(
        int(result["rotatable"]) <= 10 and tpsa <= 140
    ).lower()
    
    result["bbb_permeable"] = str(
        tpsa < 90 and logp > 0 and mw < 450
    ).lower()
    
    result["gi_absorption"] = "high" if (tpsa <= 131.6 and logp <= 5.88) else "low"
    
    return result


def hash_admet(admet: dict) -> str:
    """
    Create deterministic hash of ADMET properties.
    Properties are sorted alphabetically and concatenated.
    """
    # Sort keys for determinism
    canonical = "|".join(f"{k}={admet[k]}" for k in sorted(admet.keys()))
    return hashlib.sha256(canonical.encode()).hexdigest()


def test_determinism():
    """Verify hash is identical across runs"""
    test_smiles = [
        "CC(=O)OC1=CC=CC=C1C(=O)O",  # Aspirin
        "CC(C)CC1=CC=C(C=C1)C(C)C(=O)O",  # Ibuprofen
        "CN1C=NC2=C1C(=O)N(C(=O)N2C)C",  # Caffeine
    ]
    
    print("DETERMINISTIC ADMET HASH TEST")
    print("=" * 60)
    
    for smiles in test_smiles:
        admet = compute_deterministic_admet(smiles)
        hash1 = hash_admet(admet)
        
        # Compute again
        admet2 = compute_deterministic_admet(smiles)
        hash2 = hash_admet(admet2)
        
        mol = Chem.MolFromSmiles(smiles)
        name = smiles[:20]
        
        print(f"\n{name}...")
        print(f"  MW={admet['mw']} LogP={admet['logp']} TPSA={admet['tpsa']}")
        print(f"  Lipinski: {admet['lipinski_pass']} | QED: {admet['qed']}")
        print(f"  Hash: {hash1[:32]}...")
        print(f"  Repeat: {hash2[:32]}...")
        print(f"  Match: {'✅ YES' if hash1 == hash2 else '❌ NO'}")


if __name__ == "__main__":
    test_determinism()
    
    # Show canonical format for one compound
    print("\n" + "=" * 60)
    print("CANONICAL FORMAT (for cross-machine verification):")
    print("=" * 60)
    
    admet = compute_deterministic_admet("CC(=O)OC1=CC=CC=C1C(=O)O")
    canonical = "|".join(f"{k}={admet[k]}" for k in sorted(admet.keys()))
    print(f"\nCanonical string:\n{canonical}")
    print(f"\nSHA256: {hash_admet(admet)}")
