use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, BufReader, Write};
use stl_io::{Triangle, Vector};
use anyhow::{Result, Context};

struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    fn load(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let mesh = stl_io::read_stl(&mut reader)?;
        
        // Ensure we have complete triangles (3 vertices per triangle)
        if mesh.vertices.len() % 3 != 0 {
            return Err(anyhow::anyhow!("Invalid STL file: vertex count not divisible by 3"));
        }

        // Convert vertices to triangles
        let triangles = mesh.vertices
            .chunks_exact(3)  // Use chunks_exact to ensure we get complete triangles
            .map(|vertices| {
                Triangle {
                    normal: Vector([0.0, 0.0, 1.0]),
                    vertices: [
                        vertices[0],
                        vertices[1],
                        vertices[2]
                    ]
                }
            })
            .collect::<Vec<_>>();

        if triangles.is_empty() {
            return Err(anyhow::anyhow!("No triangles found in STL file"));
        }

        Ok(Mesh { triangles })
    }

    fn get_dimensions(&self) -> ([f32; 3], [f32; 3]) {
        let first_vertex = self.triangles[0].vertices[0];
        let mut min_array = [first_vertex[0], first_vertex[1], first_vertex[2]];
        let mut max_array = [first_vertex[0], first_vertex[1], first_vertex[2]];

        for triangle in &self.triangles {
            for vertex in &triangle.vertices {
                for i in 0..3 {
                    min_array[i] = min_array[i].min(vertex[i]);
                    max_array[i] = max_array[i].max(vertex[i]);
                }
            }
        }
        (min_array, max_array)
    }

    fn split(&self, z_height: f32) -> (Vec<Triangle>, Vec<Triangle>) {
        let mut upper = Vec::new();
        let mut lower = Vec::new();

        for triangle in &self.triangles {
            if triangle.vertices.iter().all(|v| v[2] >= z_height) {
                upper.push(*triangle);
            } else if triangle.vertices.iter().all(|v| v[2] <= z_height) {
                lower.push(*triangle);
            }
        }

        (upper, lower)
    }

    fn save(triangles: &[Triangle], path: &Path) -> Result<()> {
        let file = fs::File::create(path)?;
        stl_io::write_stl(&mut io::BufWriter::new(file), triangles.iter())?;
        Ok(())
    }
}

fn main() -> Result<()> {
    // Create directories if they don't exist
    let input_dir = PathBuf::from("models/input");
    let output_dir = PathBuf::from("models/output");
    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(&output_dir)?;

    // Get input file from command line arguments or scan directory
    let input_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| -> Option<PathBuf> {
            // If no argument provided, scan input directory
            let entries = match fs::read_dir(&input_dir) {
                Ok(entries) => entries,
                Err(e) => {
                    println!("Failed to read input directory: {}", e);
                    return None;
                }
            };
            
            let stl_files: Vec<_> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()?.to_str()? == "stl" {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect();

            if stl_files.is_empty() {
                println!("No STL files found in input directory");
                return None;
            }

            // Display available files
            println!("Available STL files:");
            for (i, path) in stl_files.iter().enumerate() {
                println!("{}. {}", i + 1, path.file_name().unwrap().to_string_lossy());
            }

            // Get user input
            print!("Select file number to process: ");
            if io::stdout().flush().is_err() {
                return None;
            }

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                return None;
            }

            let file_idx = match input.trim().parse::<usize>() {
                Ok(idx) => idx - 1,
                Err(_) => return None,
            };

            if file_idx >= stl_files.len() {
                println!("Invalid file number");
                return None;
            }

            Some(stl_files[file_idx].clone())
        })
        .context("No input file specified")?;

    println!("Loading {}...", input_path.display());
    
    // Load and process mesh
    let mesh = Mesh::load(&input_path)?;
    let (min, max) = mesh.get_dimensions();
    
    println!("Model dimensions:");
    println!("X: {:.2} to {:.2}", min[0], max[0]);
    println!("Y: {:.2} to {:.2}", min[1], max[1]);
    println!("Z: {:.2} to {:.2}", min[2], max[2]);

    let z_split = (max[2] + min[2]) / 2.0;
    println!("Splitting at Z = {:.2}", z_split);

    let (upper, lower) = mesh.split(z_split);

    // Save split parts
    let base_name = input_path.file_stem().unwrap();
    let upper_path = output_dir.join(format!("{}_upper.stl", base_name.to_string_lossy()));
    let lower_path = output_dir.join(format!("{}_lower.stl", base_name.to_string_lossy()));

    Mesh::save(&upper, &upper_path)?;
    Mesh::save(&lower, &lower_path)?;

    println!("Split complete!");
    println!("Upper part saved as: {}", upper_path.file_name().unwrap().to_string_lossy());
    println!("Lower part saved as: {}", lower_path.file_name().unwrap().to_string_lossy());

    Ok(())
}
