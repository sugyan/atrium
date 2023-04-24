mod code_writer;
mod fs;

use atprs_lex::lexicon::LexUserType;
use atprs_lex::LexiconDoc;
use code_writer::CodeWriter;
use heck::ToSnakeCase;
use itertools::Itertools;
use serde_json::from_reader;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_dir, File};
use std::io::Result;
use std::path::{Path, PathBuf};

pub fn genapi(lexdir: impl AsRef<Path>, outdir: impl AsRef<Path>, prefix: &str) -> Result<()> {
    let lexdir = lexdir.as_ref().canonicalize()?;
    let outdir = outdir.as_ref().canonicalize()?;
    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader(File::open(path)?)?);
    }
    let defmap = build_defmap(&schemas);
    for schema in schemas
        .iter()
        .filter(|schema| schema.id.starts_with(prefix))
    {
        generate_code(schema, &outdir, &defmap)?;
    }
    generate_modules(&outdir)?;
    Ok(())
}

fn build_defmap(schemas: &[LexiconDoc]) -> HashMap<String, &LexUserType> {
    let mut result = HashMap::new();
    for schema in schemas {
        for (name, def) in &schema.defs {
            let key = if name == "main" {
                schema.id.clone()
            } else {
                format!("{}#{}", schema.id, name)
            };
            assert!(!result.contains_key(&key), "duplicate key: {key}");
            result.insert(key, def);
        }
    }
    result
}

fn generate_code(
    schema: &LexiconDoc,
    outdir: &Path,
    defmap: &HashMap<String, &LexUserType>,
) -> Result<()> {
    let mut paths = schema.id.split('.').collect::<Vec<_>>();
    if let Some(name) = paths.pop() {
        create_dir_all(outdir.join(paths.join("/")))?;
        let mut writer = CodeWriter::new(Some(schema.id.clone()));
        writer.write_header(&schema.description)?;
        // TODO
        let mut keys = Vec::new();
        for (key, def) in &schema.defs {
            if key == "main" {
                writer.write_user_type(name, def, defmap, true)?;
            } else {
                keys.push(key);
            }
        }
        for &key in keys.iter().sorted() {
            let def = &schema.defs[key];
            assert!(!matches!(
                def,
                LexUserType::Record(_)
                    | LexUserType::XrpcProcedure(_)
                    | LexUserType::XrpcQuery(_)
                    | LexUserType::XrpcSubscription(_)
            ));
            writer.write_user_type(key, def, defmap, false)?;
        }
        let mut filename = PathBuf::from(name.to_snake_case());
        filename.set_extension("rs");
        writer.write_to_file(&mut File::create(
            outdir.join(paths.join("/")).join(filename),
        )?)?;
    }
    Ok(())
}

fn generate_modules(outdir: &Path) -> Result<()> {
    let paths = fs::find_dirs(outdir)?;
    let mut files = Vec::with_capacity(paths.len());
    // create ".rs" files
    for path in &paths {
        let mut p = path.to_path_buf();
        if path == outdir {
            p = p.join("lib.rs");
        } else {
            p.set_extension("rs");
        }
        files.push(File::create(&p)?);
    }
    // write "mod" statements
    for (path, mut file) in paths.iter().zip(&files) {
        let modules = read_dir(path)?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .filter_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
            })
            .sorted()
            .collect_vec();

        let mut writer = CodeWriter::new(None);
        writer.write_header(&None)?;
        writer.write_mods(&modules)?;
        writer.write_to_file(&mut file)?;
    }
    Ok(())
}
