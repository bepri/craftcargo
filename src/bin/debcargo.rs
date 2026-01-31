use clap::{crate_version, Parser};
use itertools::Itertools;
use nu_ansi_term::Color::Red;

use debcargo::cli::{Cli, Opt};
use debcargo::crates::CrateInfo;
use debcargo::debian::DebInfo;
use debcargo::errors::Result;
use debcargo::package::PackageProcess;
use debcargo::{
    build_order::build_order, crates::invalidate_crates_io_cache,
    deb_dependencies::deb_dependencies,
};

#[cfg(feature = "update-dependencies")]
use debcargo::update_dependencies::update_dependencies;
use Opt::*;

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}

fn real_main() -> Result<()> {
    let m = Cli::parse();
    match m.command {
        Update => invalidate_crates_io_cache(),
        DebSrcName {
            crate_name,
            version,
        } => {
            let crate_info = CrateInfo::new_with_update(&crate_name, version.as_deref(), false)?;
            let deb_info = DebInfo::new(&crate_info, crate_version!(), version.is_some());
            println!("{}", deb_info.package_name());
            Ok(())
        }
        Extract { init, extract } => {
            log::info!("preparing crate info");
            let mut process = PackageProcess::init(init)?;
            log::info!("extracting crate");
            process.extract(extract)?;
            Ok(())
        }
        Package {
            init,
            extract,
            finish,
        } => {
            log::info!("preparing crate info");
            let mut process = PackageProcess::init(init)?;
            log::info!("extracting crate");
            process.extract(extract)?;
            log::info!("applying overlay and patches");
            process.apply_overrides()?;
            log::info!("preparing orig tarball");
            process.prepare_orig_tarball()?;
            log::info!("preparing debian folder");
            process.prepare_debian_folder(finish)?;
            process.post_package_checks()
        }
        BuildOrder { args } => {
            let build_order = build_order(args)?;
            for v in &build_order {
                println!("{v}");
            }
            Ok(())
        }
        DebDependencies { args } => {
            let (toolchain_deps, dependencies) = deb_dependencies(args)?;
            println!(
                "{}",
                toolchain_deps.into_iter().chain(dependencies).join(", ")
            );
            Ok(())
        }
        #[cfg(feature = "update-dependencies")]
        UpdateDependencies { args } => update_dependencies(args),
    }
}

fn main() {
    env_logger::init();
    if let Err(e) = real_main() {
        eprintln!("{}", Red.bold().paint(format!("debcargo failed: {e:?}")));
        std::process::exit(1);
    }
}
