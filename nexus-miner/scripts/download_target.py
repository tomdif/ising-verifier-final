#!/usr/bin/env python3
"""
Download and prepare PDB structures for NEXUS docking.
"""

import json
import urllib.request
import subprocess
import os

def download_pdb(pdb_id: str, output_dir: str = "data/targets") -> str:
    """Download PDB file from RCSB"""
    os.makedirs(output_dir, exist_ok=True)
    url = f"https://files.rcsb.org/download/{pdb_id}.pdb"
    output_path = f"{output_dir}/{pdb_id.lower()}.pdb"
    
    if os.path.exists(output_path):
        print(f"  {pdb_id}: Already downloaded")
        return output_path
    
    print(f"  {pdb_id}: Downloading from RCSB...")
    urllib.request.urlretrieve(url, output_path)
    return output_path

def prepare_receptor(pdb_path: str) -> str:
    """Convert PDB to PDBQT using OpenBabel"""
    pdbqt_path = pdb_path.replace(".pdb", "_receptor.pdbqt")
    
    if os.path.exists(pdbqt_path):
        print(f"  {pdb_path}: PDBQT exists")
        return pdbqt_path
    
    # Extract protein only (ATOM records)
    protein_pdb = pdb_path.replace(".pdb", "_protein.pdb")
    with open(pdb_path) as f:
        lines = [l for l in f if l.startswith("ATOM")]
    with open(protein_pdb, "w") as f:
        f.writelines(lines)
    
    # Convert to PDBQT
    print(f"  Converting to PDBQT...")
    subprocess.run([
        "obabel", protein_pdb, "-O", pdbqt_path, "-xr"
    ], capture_output=True)
    
    os.remove(protein_pdb)
    return pdbqt_path

def main():
    print("=" * 60)
    print("NEXUS Target Downloader")
    print("=" * 60)
    
    with open("data/priority_targets.json") as f:
        data = json.load(f)
    
    print(f"\nDownloading {len(data['targets'])} priority targets...\n")
    
    for target in data["targets"]:
        print(f"\n{target['name']} ({target['disease']}):")
        pdb_path = download_pdb(target["pdb_id"])
        pdbqt_path = prepare_receptor(pdb_path)
        
        # Count atoms
        with open(pdbqt_path) as f:
            atoms = len([l for l in f if l.startswith("ATOM")])
        print(f"  Receptor atoms: {atoms}")
    
    print("\n" + "=" * 60)
    print("✅ All targets ready for docking!")
    print("=" * 60)

if __name__ == "__main__":
    main()
