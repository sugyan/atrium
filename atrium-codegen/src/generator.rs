use crate::code_writer::CodeWriter;
use crate::fs::find_dirs;
use crate::schema::find_ref_unions;
use atrium_lex::lexicon::LexUserType;
use atrium_lex::LexiconDoc;
use heck::ToSnakeCase;
use itertools::Itertools;
use std::fs::{create_dir_all, read_dir};
use std::io::Result;
use std::path::{Path, PathBuf};

pub(crate) fn generate_code(schema: &LexiconDoc, outdir: &Path) -> Result<()> {
    let mut paths = schema.id.split('.').collect::<Vec<_>>();
    if let Some(basename) = paths.pop() {
        create_dir_all(outdir.join(paths.join("/")))?;
        let mut writer = CodeWriter::default();
        writer.write_header(schema.description.as_ref(), Some(&schema.id))?;
        let mut names = Vec::new();
        for (name, def) in &schema.defs {
            if name == "main" {
                writer.write_user_type(&schema.id, basename, def, true)?;
            } else {
                names.push(name);
            }
        }
        for &name in names.iter().sorted() {
            let def = &schema.defs[name];
            assert!(!matches!(
                def,
                LexUserType::Record(_)
                    | LexUserType::XrpcProcedure(_)
                    | LexUserType::XrpcQuery(_)
                    | LexUserType::XrpcSubscription(_)
            ));
            writer.write_user_type(&schema.id, name, def, false)?;
        }
        writer.write_ref_unions(&schema.id, &find_ref_unions(&schema.defs))?;
        let mut filename = PathBuf::from(basename.to_snake_case());
        filename.set_extension("rs");
        writer.write_to_file(&outdir.join(paths.join("/")).join(filename))?;
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
    let mut writer = CodeWriter::default();
    writer.write_header(None, None)?;
    writer.write_records(&records)?;
    writer.write_to_file(&outdir.join("records.rs"))?;
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
    let mut writer = CodeWriter::default();
    writer.write_header(None, None)?;
    writer.write_traits_macro(&traits)?;
    writer.write_to_file(&outdir.join("traits.rs"))?;
    Ok(())
}

pub(crate) fn generate_modules(outdir: &Path) -> Result<()> {
    let paths = find_dirs(outdir)?;
    let mut files = Vec::with_capacity(paths.len());
    // create ".rs" files
    for path in &paths {
        let mut p = path.to_path_buf();
        p.set_extension("rs");
        files.push(p);
    }
    // write "mod" statements
    for (path, filepath) in paths.iter().zip(&files) {
        if path == outdir {
            continue;
        }
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

        let mut writer = CodeWriter::default();
        writer.write_header(None, None)?;
        writer.write_mods(&modules)?;
        writer.write_to_file(filepath)?;
    }
    Ok(())
}
