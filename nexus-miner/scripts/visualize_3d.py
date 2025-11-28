#!/usr/bin/env python3
"""
NEXUS 3D Visualization
Generate interactive 3D views of protein-ligand complexes
"""

import py3Dmol
import json
import os
from pathlib import Path

# Create output directory
os.makedirs("output/visualizations/3d", exist_ok=True)

# Load the receptor
receptor_pdb = "data/receptors/6LU7.pdb"
print(f"Loading receptor: {receptor_pdb}")

with open(receptor_pdb) as f:
    receptor_data = f.read()

# Load docking results
with open("data/covid_screen_results.json") as f:
    results = json.load(f)

# Check for docked poses
docked_dir = Path("data/docked_poses")
if not docked_dir.exists():
    print("No docked poses found. Creating from top result...")
    os.makedirs(docked_dir, exist_ok=True)

# Find any existing output PDBQT files
output_files = list(Path("data").glob("*_out.pdbqt")) + list(Path(".").glob("*_out.pdbqt"))
print(f"Found {len(output_files)} docked pose files")

def create_3d_view(receptor_pdb, ligand_pdbqt, title, output_file):
    """Create an interactive 3D HTML viewer"""
    
    with open(receptor_pdb) as f:
        receptor = f.read()
    
    with open(ligand_pdbqt) as f:
        ligand = f.read()
    
    # Create the HTML with embedded 3Dmol.js
    html = f'''<!DOCTYPE html>
<html>
<head>
    <title>{title} - NEXUS 3D Viewer</title>
    <script src="https://3dmol.org/build/3Dmol-min.js"></script>
    <style>
        body {{ margin: 0; padding: 20px; font-family: Arial, sans-serif; background: #1a1a2e; color: white; }}
        h1 {{ text-align: center; color: #00d4ff; }}
        #viewer {{ width: 100%; height: 600px; position: relative; }}
        .controls {{ text-align: center; margin: 20px; }}
        .controls button {{ 
            padding: 10px 20px; 
            margin: 5px;
            background: #00d4ff; 
            border: none; 
            border-radius: 5px;
            cursor: pointer;
            font-size: 14px;
        }}
        .controls button:hover {{ background: #00a8cc; }}
        .info {{ background: #16213e; padding: 15px; border-radius: 10px; margin: 20px auto; max-width: 800px; }}
    </style>
</head>
<body>
    <h1>🧬 {title}</h1>
    
    <div id="viewer"></div>
    
    <div class="controls">
        <button onclick="resetView()">Reset View</button>
        <button onclick="showSurface()">Toggle Surface</button>
        <button onclick="showCartoon()">Cartoon</button>
        <button onclick="showSticks()">Sticks</button>
        <button onclick="focusLigand()">Focus Ligand</button>
        <button onclick="spin()">Spin</button>
    </div>
    
    <div class="info">
        <h3>Controls:</h3>
        <ul>
            <li><strong>Left click + drag:</strong> Rotate</li>
            <li><strong>Right click + drag:</strong> Translate</li>
            <li><strong>Scroll:</strong> Zoom</li>
        </ul>
        <h3>Legend:</h3>
        <ul>
            <li><strong>Blue ribbon:</strong> Protein backbone</li>
            <li><strong>Colored sticks:</strong> Ligand (cyan carbon, red oxygen, blue nitrogen)</li>
            <li><strong>Yellow residues:</strong> Binding site</li>
        </ul>
    </div>
    
    <script>
        let viewer = $3Dmol.createViewer("viewer", {{backgroundColor: "0x1a1a2e"}});
        let surfaceOn = false;
        let spinning = false;
        
        // Add receptor
        viewer.addModel(`{receptor.replace("`", "'")}`, "pdb");
        
        // Add ligand  
        viewer.addModel(`{ligand.replace("`", "'")}`, "pdb");
        
        // Style protein
        viewer.setStyle({{model: 0}}, {{cartoon: {{color: "spectrum"}}}});
        
        // Style ligand
        viewer.setStyle({{model: 1}}, {{stick: {{colorscheme: "default", radius: 0.3}}}});
        
        // Highlight binding site residues (within 5A of ligand)
        viewer.setStyle({{model: 0, within: {{distance: 5, sel: {{model: 1}}}}}}, 
                        {{cartoon: {{color: "yellow"}}, stick: {{color: "yellow", radius: 0.15}}}});
        
        viewer.zoomTo();
        viewer.render();
        
        function resetView() {{
            viewer.zoomTo();
            viewer.render();
        }}
        
        function showSurface() {{
            if (surfaceOn) {{
                viewer.removeAllSurfaces();
            }} else {{
                viewer.addSurface($3Dmol.SurfaceType.VDW, {{opacity: 0.7, color: "white"}}, {{model: 0}});
            }}
            surfaceOn = !surfaceOn;
            viewer.render();
        }}
        
        function showCartoon() {{
            viewer.setStyle({{model: 0}}, {{cartoon: {{color: "spectrum"}}}});
            viewer.render();
        }}
        
        function showSticks() {{
            viewer.setStyle({{model: 0}}, {{stick: {{colorscheme: "default", radius: 0.1}}}});
            viewer.render();
        }}
        
        function focusLigand() {{
            viewer.zoomTo({{model: 1}});
            viewer.render();
        }}
        
        function spin() {{
            if (spinning) {{
                viewer.spin(false);
            }} else {{
                viewer.spin("y", 1);
            }}
            spinning = !spinning;
        }}
    </script>
</body>
</html>
'''
    
    with open(output_file, 'w') as f:
        f.write(html)
    print(f"✅ Saved: {output_file}")


# Create viewer for any docked poses we have
if output_files:
    for pdbqt in output_files[:5]:  # First 5
        name = pdbqt.stem.replace("_out", "")
        create_3d_view(
            receptor_pdb,
            str(pdbqt),
            f"COVID-19 Mpro + {name}",
            f"output/visualizations/3d/{name}_3d.html"
        )
else:
    # Run a quick dock to generate a pose
    print("\nRunning quick dock to generate 3D pose...")
    import subprocess
    
    # Use Famotidine (top binder)
    famotidine_smiles = "NC(=N)NC1=NC(CSCCC(=N)N)=CS1"
    
    # Convert to 3D
    result = subprocess.run([
        "obabel", "-:"+famotidine_smiles, "-O", "data/famotidine.pdbqt",
        "--gen3d", "-h"
    ], capture_output=True)
    
    if os.path.exists("data/famotidine.pdbqt"):
        # Run Vina
        subprocess.run([
            "./vina_1.2.5_linux_x86_64",
            "--receptor", "data/receptors/6LU7.pdbqt",
            "--ligand", "data/famotidine.pdbqt",
            "--center_x", "-10.8",
            "--center_y", "15.8", 
            "--center_z", "68.5",
            "--size_x", "20",
            "--size_y", "20",
            "--size_z", "20",
            "--out", "data/famotidine_docked.pdbqt",
            "--seed", "12345"
        ])
        
        if os.path.exists("data/famotidine_docked.pdbqt"):
            create_3d_view(
                receptor_pdb,
                "data/famotidine_docked.pdbqt",
                "COVID-19 Mpro + Famotidine (Top Binder)",
                "output/visualizations/3d/famotidine_3d.html"
            )

print("\n" + "="*50)
print("3D VISUALIZATION COMPLETE")
print("="*50)
print("\nOpen the HTML files in a browser to interact with 3D structures")
print("Files in: output/visualizations/3d/")
