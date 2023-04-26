use crate::code_writer::CodeWriter;
use crate::fs::find_dirs;
use atrium_lex::lexicon::LexUserType;
use atrium_lex::LexiconDoc;
use heck::ToSnakeCase;
use itertools::Itertools;
use std::fs::{create_dir_all, read_dir, File};
use std::io::Result;
use std::path::{Path, PathBuf};

pub(crate) fn generate_code(schema: &LexiconDoc, outdir: &Path) -> Result<()> {
    let mut paths = schema.id.split('.').collect::<Vec<_>>();
    if let Some(name) = paths.pop() {
        create_dir_all(outdir.join(paths.join("/")))?;
        let mut writer = CodeWriter::new(Some(schema.id.clone()));
        writer.write_header(&schema.description)?;
        let mut keys = Vec::new();
        for (key, def) in &schema.defs {
            if key == "main" {
                writer.write_user_type(name, def, true)?;
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
            writer.write_user_type(key, def, false)?;
        }
        let mut filename = PathBuf::from(name.to_snake_case());
        filename.set_extension("rs");
        writer.write_to_file(&mut File::create(
            outdir.join(paths.join("/")).join(filename),
        )?)?;
    }
    Ok(())
}

pub(crate) fn generate_records(outdir: &Path, schemas: &[LexiconDoc]) -> Result<()> {
    let records = schemas
        .iter()
        .filter_map(|schema| {
            if let Some(LexUserType::Record(_)) = schema.defs.get("main") {
                Some(schema.id.clone())
            } else {
                None
            }
        })
        .sorted()
        .collect_vec();
    let mut writer = CodeWriter::new(None);
    writer.write_header(&Some(String::from(
        "Collection of ATP repository record type",
    )))?;
    writer.write_records(&records)?;
    writer.write_to_file(&mut File::create(outdir.join("records.rs"))?)?;
    Ok(())
}

pub(crate) fn generate_traits(outdir: &Path, schemas: &[LexiconDoc]) -> Result<()> {
    let traits = schemas
        .iter()
        .filter_map(|schema| {
            if let Some(LexUserType::XrpcQuery(_) | LexUserType::XrpcProcedure(_)) =
                schema.defs.get("main")
            {
                Some(schema.id.clone())
            } else {
                None
            }
        })
        .sorted()
        .collect_vec();
    let mut writer = CodeWriter::new(None);
    writer.write_header(&None)?;
    writer.write_traits_macro(&traits)?;
    writer.write_to_file(&mut File::create(outdir.join("traits.rs"))?)?;
    Ok(())
}

pub(crate) fn generate_modules(outdir: &Path) -> Result<()> {
    let paths = find_dirs(outdir)?;
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
