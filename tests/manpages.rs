use clap::CommandFactory;
use clap_mangen::generate_to;
use std::{fs, path::PathBuf};

use debcargo::cli::Cli;

#[test]
fn check_manpages() {
    let outdir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("manpages");
    let manpages_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manpages");

    fs::remove_dir_all(&outdir).ok();
    fs::create_dir_all(&outdir)
        .unwrap_or_else(|e| panic!("Could not create temporary manpages directory: {e}"));

    // Generate manpages from scratch
    generate_to(Cli::command().flatten_help(true), &outdir)
        .unwrap_or_else(|e| panic!("Could not generate temporary manpages: {e}"));

    // For each generated manpage, check that it exists with the same content in the source directory
    let changed_manpages: Vec<String> = fs::read_dir(&outdir)
        .unwrap_or_else(|e| panic!("Could not read temporary manpages directory: {e}"))
        .filter(|manpage| {
            let new_manpage = manpage.as_ref().unwrap().path();
            let old_manpage = manpages_dir.join(new_manpage.file_name().unwrap());
            if !old_manpage.exists() {
                true
            } else {
                let old_manpage_content = fs::read_to_string(&old_manpage).unwrap_or_else(|e| {
                    panic!("Could not read {}: {e}", old_manpage.to_string_lossy())
                });
                let new_manpage_content = fs::read_to_string(&new_manpage).unwrap_or_else(|e| {
                    panic!("Could not read {}: {e}", new_manpage.to_string_lossy())
                });
                old_manpage_content != new_manpage_content
            }
        })
        .map(|manpage| manpage.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    // For each manpage in the source directory, check that the generated one exists (i.e. whether
    // it should be removed)
    let removed_manpages: Vec<String> = fs::read_dir(&manpages_dir)
        .unwrap()
        .filter(|manpage| {
            !outdir
                .join(manpage.as_ref().unwrap().path().file_name().unwrap())
                .exists()
        })
        .map(|manpage| manpage.unwrap().file_name().to_string_lossy().to_string())
        .collect();

    if !(changed_manpages.is_empty() && removed_manpages.is_empty()) {
        let mut panic_string = vec![String::from("Manpages have changed!")];
        if !changed_manpages.is_empty() {
            panic_string.push(format!(
                "- Please replace {} (new manpages generated in {})",
                changed_manpages.join(", "),
                outdir.to_string_lossy()
            ));
        }
        if !removed_manpages.is_empty() {
            panic_string.push(format!("- Please delete {}", removed_manpages.join(", "),));
        }
        panic!("{}", panic_string.join("\n"));
    }
}
