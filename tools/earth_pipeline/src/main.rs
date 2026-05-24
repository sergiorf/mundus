mod manifest;

use geo::Contains;
use geo::{MultiPolygon, Point};
use image::imageops::FilterType;
use manifest::{Manifest, Source};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use zip::ZipArchive;

fn main() {
    if let Err(error) = run() {
        eprintln!("earth_pipeline: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");

    match command {
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "verify" => verify_inputs(),
        "scaffold-first-milestone" => scaffold_first_milestone(),
        "bake-first-milestone" => bake_first_milestone(),
        other => Err(format!("unknown command `{other}`").into()),
    }
}

fn print_help() {
    println!("Usage:");
    println!("  cargo run -p earth_pipeline -- verify");
    println!("  cargo run -p earth_pipeline -- scaffold-first-milestone");
    println!("  cargo run -p earth_pipeline -- bake-first-milestone");
}

fn verify_inputs() -> Result<(), Box<dyn Error>> {
    let repo_root = repo_root()?;
    let manifest = load_manifest(&repo_root)?;
    let required_ids = manifest.first_milestone.required_sources.clone();
    let required_sources = manifest.sources_for_ids(&required_ids)?;

    println!("First milestone goal:");
    println!("  {}", manifest.first_milestone.goal);
    println!();

    for source in &required_sources {
        let matches = find_source_files(&repo_root, source)?;
        println!("{}:", source.id);
        println!("  provider: {}", source.provider);
        println!("  cache dir: {}", source.local_cache_dir);
        println!("  found files: {}", matches.len());
        for path in matches {
            println!("  - {}", relative_to_repo(&repo_root, &path).display());
        }
        println!();
    }

    Ok(())
}

fn scaffold_first_milestone() -> Result<(), Box<dyn Error>> {
    let repo_root = repo_root()?;
    let manifest = load_manifest(&repo_root)?;
    let required_sources = manifest.sources_for_ids(&manifest.first_milestone.required_sources)?;

    let natural_earth = required_sources
        .iter()
        .find(|source| source.id == "natural_earth_110m_physical")
        .ok_or("missing Natural Earth source in manifest")?;
    let blue_marble = required_sources
        .iter()
        .find(|source| source.id == "nasa_blue_marble_next_generation")
        .ok_or("missing Blue Marble source in manifest")?;

    let natural_earth_files = find_source_files(&repo_root, natural_earth)?;
    if natural_earth_files.is_empty() {
        return Err("no Natural Earth files found for the first milestone".into());
    }

    let blue_marble_files = find_source_files(&repo_root, blue_marble)?;
    if blue_marble_files.is_empty() {
        return Err("no Blue Marble files found for the first milestone".into());
    }

    let target_dir = repo_root.join("assets/earth/raw/first_milestone");
    fs::create_dir_all(&target_dir)?;
    fs::create_dir_all(repo_root.join("assets/earth/render/lod0"))?;
    fs::create_dir_all(repo_root.join("assets/earth/sim"))?;

    let sim_output = repo_root.join("assets/earth/sim/earth_360x180_landmask.toml");
    let render_output = repo_root.join("assets/earth/render/lod0");
    let plan_path = target_dir.join("targets.toml");

    let natural_earth_lines = natural_earth_files
        .iter()
        .map(|path| format!("  \"{}\"", relative_to_repo(&repo_root, path).display()))
        .collect::<Vec<_>>()
        .join(",\n");
    let blue_marble_lines = blue_marble_files
        .iter()
        .map(|path| format!("  \"{}\"", relative_to_repo(&repo_root, path).display()))
        .collect::<Vec<_>>()
        .join(",\n");

    let plan = format!(
        "version = 1\n\n\
         [first_milestone]\n\
         goal = \"{goal}\"\n\n\
         [inputs]\n\
         natural_earth = [\n{natural_earth}\n]\n\
         blue_marble = [\n{blue_marble}\n]\n\n\
         [outputs]\n\
         sim_landmask = \"{sim_output}\"\n\
         render_lod0_dir = \"{render_output}\"\n\n\
         [notes]\n\
         next_step = \"Implement a bake command that rasterizes Natural Earth land polygons into the simulation Earth mask and cuts Blue Marble into LOD0 color tiles.\"\n",
        goal = manifest.first_milestone.goal,
        natural_earth = natural_earth_lines,
        blue_marble = blue_marble_lines,
        sim_output = relative_to_repo(&repo_root, &sim_output).display(),
        render_output = relative_to_repo(&repo_root, &render_output).display(),
    );
    fs::write(&plan_path, plan)?;

    println!("Scaffolded first milestone plan:");
    println!("  {}", relative_to_repo(&repo_root, &plan_path).display());
    println!("  {}", relative_to_repo(&repo_root, &sim_output).display());
    println!(
        "  {}",
        relative_to_repo(&repo_root, &render_output).display()
    );

    Ok(())
}

fn bake_first_milestone() -> Result<(), Box<dyn Error>> {
    let repo_root = repo_root()?;
    let manifest = load_manifest(&repo_root)?;
    let required_sources = manifest.sources_for_ids(&manifest.first_milestone.required_sources)?;

    let natural_earth = required_sources
        .iter()
        .find(|source| source.id == "natural_earth_110m_physical")
        .ok_or("missing Natural Earth source in manifest")?;
    let blue_marble = required_sources
        .iter()
        .find(|source| source.id == "nasa_blue_marble_next_generation")
        .ok_or("missing Blue Marble source in manifest")?;

    let natural_earth_zip = single_required_file(&repo_root, natural_earth)?;
    let blue_marble_png = single_required_file(&repo_root, blue_marble)?;

    let sim_output = repo_root.join("assets/earth/sim/earth_360x180_landmask.toml");
    let lod0_dir = repo_root.join("assets/earth/render/lod0");
    fs::create_dir_all(sim_output.parent().ok_or("missing sim output parent")?)?;
    fs::create_dir_all(&lod0_dir)?;

    let polygons = load_land_polygons_from_zip(&natural_earth_zip)?;
    write_landmask_toml(
        &sim_output,
        &repo_root,
        &natural_earth_zip,
        &polygons,
        360,
        180,
    )?;
    write_lod0_tiles(&blue_marble_png, &repo_root, &lod0_dir)?;

    println!("Baked first milestone outputs:");
    println!("  {}", relative_to_repo(&repo_root, &sim_output).display());
    println!("  {}", relative_to_repo(&repo_root, &lod0_dir).display());

    Ok(())
}

fn repo_root() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or("failed to determine repo root")?;
    Ok(repo_root.to_path_buf())
}

fn load_manifest(repo_root: &Path) -> Result<Manifest, Box<dyn Error>> {
    let manifest_path = repo_root.join("tools/earth_pipeline/manifest.toml");
    let contents = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = toml::from_str(&contents)?;
    Ok(manifest)
}

fn single_required_file(repo_root: &Path, source: &Source) -> Result<PathBuf, Box<dyn Error>> {
    let matches = find_source_files(repo_root, source)?;
    match matches.as_slice() {
        [path] => Ok(path.clone()),
        [] => Err(format!("no files found for source `{}`", source.id).into()),
        _ => Err(format!(
            "expected one file for source `{}`, found {}",
            source.id,
            matches.len()
        )
        .into()),
    }
}

fn find_source_files(repo_root: &Path, source: &Source) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let cache_dir = repo_root.join(&source.local_cache_dir);
    let mut matches = Vec::new();
    if !cache_dir.exists() {
        return Ok(matches);
    }

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if source.matches_file_name(file_name) {
            matches.push(path);
        }
    }

    matches.sort();
    Ok(matches)
}

fn relative_to_repo<'a>(repo_root: &'a Path, path: &'a Path) -> PathBuf {
    path.strip_prefix(repo_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn load_land_polygons_from_zip(zip_path: &Path) -> Result<Vec<MultiPolygon<f64>>, Box<dyn Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let temp_dir = tempdir()?;
    let needed = [
        "ne_110m_land.shp",
        "ne_110m_land.shx",
        "ne_110m_land.dbf",
        "ne_110m_land.prj",
        "ne_110m_land.cpg",
    ];

    for name in needed {
        let mut zipped = archive.by_name(name)?;
        let out_path = temp_dir.path().join(name);
        let mut out_file = fs::File::create(out_path)?;
        io::copy(&mut zipped, &mut out_file)?;
    }

    let shapefile_path = temp_dir.path().join("ne_110m_land.shp");
    let mut reader = shapefile::Reader::from_path(shapefile_path)?;
    let mut polygons = Vec::new();

    for shape_record in reader.iter_shapes_and_records() {
        let (shape, _) = shape_record?;
        let geometry: geo_types::Geometry<f64> = shape.try_into()?;
        match geometry {
            geo_types::Geometry::Polygon(polygon) => polygons.push(MultiPolygon(vec![polygon])),
            geo_types::Geometry::MultiPolygon(multi_polygon) => polygons.push(multi_polygon),
            _ => {}
        }
    }

    Ok(polygons)
}

fn write_landmask_toml(
    output_path: &Path,
    repo_root: &Path,
    source_zip: &Path,
    polygons: &[MultiPolygon<f64>],
    width: usize,
    height: usize,
) -> Result<(), Box<dyn Error>> {
    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let lat = 90.0 - (y as f64 + 0.5) * 180.0 / height as f64;
        let mut row = String::with_capacity(width);
        for x in 0..width {
            let lon = -180.0 + (x as f64 + 0.5) * 360.0 / width as f64;
            let point = Point::new(lon, lat);
            let is_land = polygons.iter().any(|polygon| polygon.contains(&point));
            row.push(if is_land { '#' } else { '.' });
        }
        rows.push(row);
    }

    let rows_block = rows
        .iter()
        .map(|row| format!("  \"{row}\""))
        .collect::<Vec<_>>()
        .join(",\n");

    let content = format!(
        "version = 1\n\
         width = {width}\n\
         height = {height}\n\
         source = \"{source}\"\n\n\
         rows = [\n{rows}\n]\n",
        width = width,
        height = height,
        source = relative_to_repo(repo_root, source_zip).display(),
        rows = rows_block
    );
    fs::write(output_path, content)?;

    Ok(())
}

fn write_lod0_tiles(
    source_png: &Path,
    repo_root: &Path,
    lod0_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let image = image::open(source_png)?;
    let resized = image.resize_exact(512, 256, FilterType::CatmullRom);

    for tile_x in 0..2u32 {
        let tile = resized.crop_imm(tile_x * 256, 0, 256, 256);
        let tile_path = lod0_dir.join(format!("tile_{tile_x}_0.png"));
        tile.save(tile_path)?;
    }

    let manifest_path = lod0_dir.join("manifest.toml");
    let content = format!(
        "version = 1\n\
         tile_size = 256\n\
         tiles_x = 2\n\
         tiles_y = 1\n\
         source = \"{}\"\n\
         source_dimensions = [{}, {}]\n\n\
         [[tiles]]\n\
         x = 0\n\
         y = 0\n\
         path = \"tile_0_0.png\"\n\n\
         [[tiles]]\n\
         x = 1\n\
         y = 0\n\
         path = \"tile_1_0.png\"\n",
        relative_to_repo(repo_root, source_png).display(),
        image.width(),
        image.height()
    );
    fs::write(manifest_path, content)?;

    Ok(())
}
