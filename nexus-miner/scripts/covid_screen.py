#!/usr/bin/env python3
"""
NEXUS COVID-19 Drug Repurposing Screen
Screens FDA-approved drugs against SARS-CoV-2 Main Protease (Mpro)
"""

import subprocess
import json
import os
import sys
from pathlib import Path

# Add scripts to path
sys.path.insert(0, str(Path(__file__).parent))
from deterministic_admet import compute_deterministic_admet, hash_admet

# FDA-approved drugs with known SMILES (subset for demo)
# These are real drugs currently on the market
FDA_DRUGS = [
    ("Nirmatrelvir", "CC1(C2C1C(N(C2)C(=O)C(C(C)(C)C)NC(=O)C(F)(F)F)C(=O)NC(CC3CCNC3=O)C#N)C", "COVID antiviral (Paxlovid)"),
    ("Remdesivir", "CCC(CC)COC(=O)C(C)NP(=O)(OCC1C(C(C(O1)N2C=CC(=O)NC2=O)(C)O)O)OC3=CC=CC=C3", "COVID antiviral"),
    ("Lopinavir", "CC1=C(C=CC=C1)C(CC(=O)NC(CC2=CC=CC=C2)C(CC(CC3=CC=CC=C3)NC(=O)C(C(C)C)N4CCCNC4=O)O)NC(=O)C(C(C)C)N5CCCNC5=O", "HIV protease inhibitor"),
    ("Ritonavir", "CC(C)C(NC(=O)N(C)CC1=CSC(=N1)C(C)C)C(=O)NC(CC(O)C(CC2=CC=CC=C2)NC(=O)OCC3=CN=CS3)CC4=CC=CC=C4", "HIV protease inhibitor"),
    ("Darunavir", "CC(C)CN(CC(C(CC1=CC=CC=C1)NC(=O)OC2COC3C2CCO3)O)S(=O)(=O)C4=CC=C(C=C4)N", "HIV protease inhibitor"),
    ("Nelfinavir", "CC1=C(C=CC=C1O)C(=O)NC(CSC2=CC=CC=C2)C(CN3CC4CCCCC4CC3C(=O)NC(C)(C)C)O", "HIV protease inhibitor"),
    ("Saquinavir", "CC(C)(C)NC(=O)C1CC2CCCCC2CN1CC(C(CC3=CC=CC=C3)NC(=O)C(CC(=O)N)NC(=O)C4=NC5=CC=CC=C5C=C4)O", "HIV protease inhibitor"),
    ("Atazanavir", "COC(=O)NC(C(C)C)C(=O)NC(CC1=CC=CC=C1)C(CN(CC2=CC=C(C=C2)C3=CC=CC=N3)NC(=O)C(C(C)C)NC(=O)OC)O", "HIV protease inhibitor"),
    ("Boceprevir", "CC1(C2C1C(N(C2)C(=O)C(C(C)(C)C)NC(=O)NC(C)(C)C)C(=O)NC(CC3CCC(=O)NC3)C(=O)C(=O)N)C", "HCV protease inhibitor"),
    ("Telaprevir", "CCCC(C(=O)C(=O)NC1CC1)NC(=O)C2C3CCCC3CN2C(=O)C(C(C)C)NC(=O)C(C4CCCCC4)NC(=O)C5=NC=CN=C5", "HCV protease inhibitor"),
    ("Simeprevir", "COC1=C2C=CC=C(C2=CC(=C1)OC)C(=O)NC3CC4C(C3)C(=O)N(C(=CC=CC5CC5)C(=O)NC6(CCCCC6)C(=O)NS(=O)(=O)C7CC7)C4", "HCV protease inhibitor"),
    ("Chloroquine", "CCN(CC)CCCC(C)NC1=C2C=CC(=CC2=NC=C1)Cl", "Antimalarial"),
    ("Hydroxychloroquine", "CCN(CCO)CCCC(C)NC1=C2C=CC(=CC2=NC=C1)Cl", "Antimalarial"),
    ("Ivermectin", "CCC(C)C(C)C=CC=CC(OC1CC(OC2CC(OC(CC(C(C(C(=O)O2)C)OC3CC(C(C(O3)C)O)OC)(C)C)C)OC(C(C1O)C)CC4C(O4)C)C)C", "Antiparasitic"),
    ("Favipiravir", "NC1=NC(=O)C(=CN1)F", "Antiviral (flu)"),
    ("Ribavirin", "C1=NC(=NN1C2C(C(C(O2)CO)O)O)C(=O)N", "Antiviral (HCV)"),
    ("Oseltamivir", "CCOC(=O)C1=CC(C(C(C1)N)OC(CC)CC)NC(C)=O", "Antiviral (flu)"),
    ("Camostat", "CN(C)C(=O)COC(=O)CC1=CC=C(C=C1)OC(=O)C2=CC=C(C=C2)N=C(N)N", "Serine protease inhibitor"),
    ("Nafamostat", "N=C(N)C1=CC=C(C=C1)C(=O)OC2=CC3=C(C=C2)C=C(C=C3)C(=O)N", "Serine protease inhibitor"),
    ("Dexamethasone", "CC1CC2C3CCC4=CC(=O)C=CC4(C3(C(CC2(C1(C(=O)CO)O)C)O)F)C", "Corticosteroid"),
    ("Baricitinib", "CCS(=O)(=O)N1CC(C1)N2C=C(C=N2)C3=C4C=CNC4=NC=N3", "JAK inhibitor"),
    ("Tocilizumab_proxy", "CC(C)CC(NC(=O)C(CC1=CC=CC=C1)NC(=O)C(CC(=O)O)NC(=O)C)C(=O)O", "IL-6 inhibitor (small molecule proxy)"),
    ("Molnupiravir", "CC(C)C(=O)OCC1C(C(C(O1)N2C=CC(=NC2=O)NO)O)O", "COVID antiviral"),
    ("Paxlovid_ritonavir", "CC(C)C(NC(=O)N(C)CC1=CSC(=N1)C(C)C)C(=O)NC(CC(O)C(CC2=CC=CC=C2)NC(=O)OCC3=CN=CS3)CC4=CC=CC=C4", "Paxlovid component"),
    ("Aspirin", "CC(=O)OC1=CC=CC=C1C(=O)O", "NSAID"),
    ("Ibuprofen", "CC(C)CC1=CC=C(C=C1)C(C)C(=O)O", "NSAID"),
    ("Naproxen", "COC1=CC2=CC(C(C)C(=O)O)=CC=C2C=C1", "NSAID"),
    ("Indomethacin", "COC1=CC2=C(C=C1)C(=C(N2C(=O)C3=CC=C(C=C3)Cl)CC(=O)O)C", "NSAID"),
    ("Colchicine", "COC1=C(C=C2C(=C1)C(CC3=CC(=C(C(=C3C2=O)OC)OC)OC)NC(=O)C)OC", "Anti-inflammatory"),
    ("Famotidine", "NC(=N)NC1=NC(CSCCC(=N)NS(=O)(=O)N)=CS1", "H2 blocker"),
    ("Omeprazole", "COC1=CC2=C(C=C1)N=C(S2)S(=O)CC3=NC=C(C=C3C)OC", "Proton pump inhibitor"),
    ("Metformin", "CN(C)C(=N)NC(=N)N", "Diabetes"),
    ("Atorvastatin", "CC(C)C1=C(C(=C(N1CCC(CC(CC(=O)O)O)O)C2=CC=C(C=C2)F)C3=CC=CC=C3)C(=O)NC4=CC=CC=C4", "Statin"),
    ("Losartan", "CCCCC1=NC(=C(N1CC2=CC=C(C=C2)C3=CC=CC=C3C4=NNN=N4)CO)Cl", "ARB"),
    ("Captopril", "CC(CS)C(=O)N1CCCC1C(=O)O", "ACE inhibitor"),
    ("Lisinopril", "NCCCC(NC(CCC1=CC=CC=C1)C(=O)O)C(=O)N2CCCC2C(=O)O", "ACE inhibitor"),
    ("Azithromycin", "CCC1C(C(C(N(CC(CC(C(C(C(C(C(=O)O1)C)OC2CC(C(C(O2)C)O)(C)OC)C)OC3C(C(CC(O3)C)N(C)C)O)(C)O)C)C)C)O)(C)O", "Antibiotic"),
    ("Doxycycline", "CC1C2C(C3C(C(=O)C(=C(C3(C(=O)C2=C(C4=C1C=CC=C4O)O)O)O)C(=O)N)N(C)C)O", "Antibiotic"),
    ("Nitazoxanide", "CC(=O)OC1=CC=CC=C1C(=O)NC2=NC=C(S2)[N+](=O)[O-]", "Antiparasitic"),
    ("Fluvoxamine", "COCCCC/C(=N\\OCCN)C1=CC=C(C=C1)C(F)(F)F", "SSRI"),
]

def smiles_to_pdbqt(smiles: str, name: str, output_dir: str) -> str:
    """Convert SMILES to PDBQT using OpenBabel"""
    output_path = f"{output_dir}/{name}.pdbqt"
    
    if os.path.exists(output_path):
        return output_path
    
    # Use obabel to convert SMILES to 3D PDBQT
    cmd = f'obabel -:"{smiles}" -O {output_path} --gen3d -h 2>/dev/null'
    result = subprocess.run(cmd, shell=True, capture_output=True)
    
    if os.path.exists(output_path) and os.path.getsize(output_path) > 0:
        return output_path
    return None

def run_docking(receptor: str, ligand: str, center: tuple, size: tuple, seed: int) -> dict:
    """Run Vina docking and return results"""
    output_path = "/tmp/dock_result.pdbqt"
    
    cmd = [
        "./vina_1.2.5_linux_x86_64",
        "--receptor", receptor,
        "--ligand", ligand,
        "--center_x", str(center[0]),
        "--center_y", str(center[1]),
        "--center_z", str(center[2]),
        "--size_x", str(size[0]),
        "--size_y", str(size[1]),
        "--size_z", str(size[2]),
        "--seed", str(seed),
        "--exhaustiveness", "8",
        "--num_modes", "1",
        "--out", output_path,
    ]
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    # Parse affinity
    affinity = None
    for line in result.stdout.split('\n'):
        parts = line.split()
        if len(parts) >= 2 and parts[0] == '1':
            try:
                affinity = float(parts[1])
            except:
                pass
            break
    
    return {"affinity": affinity, "stdout": result.stdout, "stderr": result.stderr}

def main():
    print("=" * 70)
    print("NEXUS COVID-19 DRUG REPURPOSING SCREEN")
    print("Target: SARS-CoV-2 Main Protease (Mpro) - PDB: 6LU7")
    print("=" * 70)
    
    # Setup paths
    receptor = "data/targets/6lu7_receptor.pdbqt"
    ligand_dir = "data/ligands"
    os.makedirs(ligand_dir, exist_ok=True)
    
    # COVID Mpro binding site
    center = (-10.8, 15.8, 68.5)
    size = (20, 20, 20)
    
    results = []
    
    print(f"\nScreening {len(FDA_DRUGS)} FDA-approved drugs...\n")
    
    for i, (name, smiles, category) in enumerate(FDA_DRUGS):
        print(f"[{i+1}/{len(FDA_DRUGS)}] {name}...", end=" ", flush=True)
        
        # Convert to PDBQT
        ligand_path = smiles_to_pdbqt(smiles, name.replace(" ", "_"), ligand_dir)
        
        if not ligand_path:
            print("❌ conversion failed")
            continue
        
        # Generate deterministic seed
        seed = hash(f"covid-mpro-{name}") & 0x7FFFFFFF
        
        # Run docking
        dock_result = run_docking(receptor, ligand_path, center, size, seed)
        
        if dock_result["affinity"] is None:
            print("❌ docking failed")
            continue
        
        # Compute ADMET
        admet = compute_deterministic_admet(smiles)
        
        if admet is None:
            print("❌ ADMET failed")
            continue
        
        # Combine results
        result = {
            "rank": 0,
            "name": name,
            "category": category,
            "smiles": smiles,
            "affinity": dock_result["affinity"],
            "mw": float(admet["mw"]),
            "logp": float(admet["logp"]),
            "qed": float(admet["qed"]),
            "lipinski": admet["lipinski_pass"] == "true",
            "bbb": admet["bbb_permeable"] == "true",
            "gi": admet["gi_absorption"],
        }
        
        results.append(result)
        print(f"✅ {dock_result['affinity']:.2f} kcal/mol")
    
    # Sort by affinity (more negative = better binding)
    results.sort(key=lambda x: x["affinity"])
    
    # Assign ranks
    for i, r in enumerate(results):
        r["rank"] = i + 1
    
    # Print results table
    print("\n" + "=" * 70)
    print("RESULTS - Ranked by Binding Affinity")
    print("=" * 70)
    print(f"{'Rank':<5} {'Drug':<20} {'Affinity':<10} {'QED':<6} {'Lipinski':<8} {'Category'}")
    print("-" * 70)
    
    for r in results[:20]:  # Top 20
        lip = "✅" if r["lipinski"] else "❌"
        print(f"{r['rank']:<5} {r['name'][:19]:<20} {r['affinity']:<10.2f} {r['qed']:<6.2f} {lip:<8} {r['category'][:20]}")
    
    # Save full results
    output_file = "data/covid_screen_results.json"
    with open(output_file, "w") as f:
        json.dump(results, f, indent=2)
    
    print(f"\n✅ Full results saved to {output_file}")
    
    # Summary statistics
    print("\n" + "=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print(f"Total drugs screened: {len(results)}")
    print(f"Best binder: {results[0]['name']} ({results[0]['affinity']:.2f} kcal/mol)")
    print(f"Worst binder: {results[-1]['name']} ({results[-1]['affinity']:.2f} kcal/mol)")
    
    # Known COVID drugs performance
    covid_drugs = [r for r in results if "COVID" in r["category"] or "Paxlovid" in r["category"]]
    if covid_drugs:
        print(f"\nKnown COVID antivirals:")
        for d in covid_drugs:
            print(f"  {d['name']}: rank {d['rank']}, {d['affinity']:.2f} kcal/mol")

if __name__ == "__main__":
    main()
