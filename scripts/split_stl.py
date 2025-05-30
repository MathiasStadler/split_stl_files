import numpy as np
from stl import mesh
import math
import os

def load_stl(filename):
    """Load STL file and return mesh object"""
    try:
        return mesh.Mesh.from_file(filename)
    except FileNotFoundError:
        raise Exception(f"File {filename} not found")

def get_dimensions(mesh_obj):
    """Get min and max dimensions of the mesh"""
    minx = maxx = mesh_obj.vectors[0][0][0]
    miny = maxy = mesh_obj.vectors[0][0][1]
    minz = maxz = mesh_obj.vectors[0][0][2]

    for vector in mesh_obj.vectors:
        for vertex in vector:
            minx = min(minx, vertex[0])
            maxx = max(maxx, vertex[0])
            miny = min(miny, vertex[1])
            maxy = max(maxy, vertex[1])
            minz = min(minz, vertex[2])
            maxz = max(maxz, vertex[2])
    
    return (minx, maxx, miny, maxy, minz, maxz)

def split_mesh(mesh_obj, z_height):
    """Split mesh at given Z height"""
    # Get vertices above and below split point
    upper_triangles = []
    lower_triangles = []
    
    for triangle in mesh_obj.vectors:
        if all(vertex[2] >= z_height for vertex in triangle):
            upper_triangles.append(triangle)
        elif all(vertex[2] <= z_height for vertex in triangle):
            lower_triangles.append(triangle)
    
    # Create new meshes
    upper = mesh.Mesh(np.zeros(len(upper_triangles), dtype=mesh.Mesh.dtype))
    lower = mesh.Mesh(np.zeros(len(lower_triangles), dtype=mesh.Mesh.dtype))
    
    for i, triangle in enumerate(upper_triangles):
        upper.vectors[i] = triangle
    for i, triangle in enumerate(lower_triangles):
        lower.vectors[i] = triangle
        
    return upper, lower

def main():
    input_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "models", "input")
    output_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), "models", "output")
    
    # List STL files in input directory
    stl_files = [f for f in os.listdir(input_dir) if f.endswith('.stl')]
    
    if not stl_files:
        print("No STL files found in input directory")
        return
        
    print("Available STL files:")
    for i, file in enumerate(stl_files):
        print(f"{i+1}. {file}")
        
    file_idx = int(input("Select file number to process: ")) - 1
    input_file = os.path.join(input_dir, stl_files[file_idx])
    
    try:
        # Load the STL file
        print(f"Loading {stl_files[file_idx]}...")
        model = load_stl(input_file)
        
        # Get model dimensions
        minx, maxx, miny, maxy, minz, maxz = get_dimensions(model)
        print(f"Model dimensions:")
        print(f"X: {minx:.2f} to {maxx:.2f}")
        print(f"Y: {miny:.2f} to {maxy:.2f}")
        print(f"Z: {minz:.2f} to {maxz:.2f}")
        
        # Calculate middle point for splitting
        z_split = (maxz + minz) / 2
        print(f"Splitting at Z = {z_split:.2f}")
        
        # Split the model
        upper, lower = split_mesh(model, z_split)
        
        # Save the parts
        base_name = os.path.splitext(stl_files[file_idx])[0]
        upper_file = os.path.join(output_dir, f"{base_name}_upper.stl")
        lower_file = os.path.join(output_dir, f"{base_name}_lower.stl")
        
        upper.save(upper_file)
        lower.save(lower_file)
        
        print("Split complete!")
        print(f"Upper part saved as: {os.path.basename(upper_file)}")
        print(f"Lower part saved as: {os.path.basename(lower_file)}")
        
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()