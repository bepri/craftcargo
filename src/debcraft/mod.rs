pub mod schema;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::crates::CrateInfo;
use crate::debian::DebInfo;
use crate::errors::Result;

use schema::{DebcraftPackage, DebcraftPart, DebcraftYaml};

/// Generate a `debcraft.yaml` file and companion files in `output_dir`.
///
/// This is the debcraft equivalent of `prepare_debian_folder()`.  Instead of
/// writing a full `debian/` tree it builds the schema in memory, serializes it
/// to YAML, and writes a small set of companion files alongside it.
#[allow(clippy::too_many_arguments)]
pub fn prepare_debcraft_yaml(
    crate_info: &mut CrateInfo,
    deb_info: &DebInfo,
    config_path: Option<&Path>,
    config: &Config,
    output_dir: &Path,
    copyright_guess_harder: bool,
) -> Result<()> {
    let mut yaml = build_debcraft_top_level(crate_info, deb_info, config)?;

    yaml.packages = build_debcraft_packages(deb_info, crate_info, config)?;

    yaml.parts = build_debcraft_parts(crate_info, deb_info, config, &yaml.packages)?;

    apply_source_overrides(&mut yaml, config);

    fs::create_dir_all(output_dir)?;
    let yaml_str = serde_yaml_ng::to_string(&yaml)?;
    fs::write(output_dir.join("debcraft.yaml"), yaml_str)?;

    write_companion_files(
        crate_info,
        deb_info,
        config_path,
        config,
        copyright_guess_harder,
        output_dir,
    )?;

    Ok(())
}

fn build_debcraft_top_level(
    _crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    _config: &Config,
) -> Result<DebcraftYaml> {
    todo!("step 5: top-level field mapping")
}

fn build_debcraft_packages(
    _deb_info: &DebInfo,
    _crate_info: &CrateInfo,
    _config: &Config,
) -> Result<BTreeMap<String, DebcraftPackage>> {
    todo!("step 6: binary package generation")
}

fn build_debcraft_parts(
    _crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    _config: &Config,
    _packages: &BTreeMap<String, DebcraftPackage>,
) -> Result<BTreeMap<String, DebcraftPart>> {
    todo!("step 7: parts generation")
}

fn apply_source_overrides(_yaml: &mut DebcraftYaml, _config: &Config) {
    todo!("step 5: source overrides")
}

fn write_companion_files(
    _crate_info: &CrateInfo,
    _deb_info: &DebInfo,
    _config_path: Option<&Path>,
    _config: &Config,
    _copyright_guess_harder: bool,
    _out_dir: &Path,
) -> Result<()> {
    todo!("step 8: companion files")
}
