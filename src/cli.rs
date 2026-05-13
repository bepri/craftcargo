use clap::{builder::styling::AnsiColor, builder::Styles, Parser, Subcommand};

use crate::{
    build_order::BuildOrderArgs,
    deb_dependencies::DebDependenciesArgs,
    package::{PackageDebcraftArgs, PackageExecuteArgs, PackageExtractArgs, PackageInitArgs},
};

#[cfg(feature = "update-dependencies")]
use crate::update_dependencies::UpdateDependenciesArgs;

const CLI_STYLE: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default())
    .usage(AnsiColor::Green.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Green.on_default());

#[derive(Debug, Clone, Parser)]
#[command(name = "debcargo", about = "Package Rust crates for Debian.")]
#[command(version)]
#[command(styles = CLI_STYLE)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Opt,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Opt {
    /// Update the user's default crates.io index, outside of a workspace.
    Update,
    /// Print the Debian package name for a crate.
    DebSrcName {
        /// Name of the crate to package.
        crate_name: String,
        /// Version of the crate to package; may contain dependency operators.
        /// If empty string, resolves to the latest version. If given here,
        /// i.e. not omitted then print the package name as if the config
        /// option `semver_suffix` was set to true.
        version: Option<String>,
    },
    /// Extract only a crate, without any other transformations.
    Extract {
        #[command(flatten)]
        init: PackageInitArgs,
        #[command(flatten)]
        extract: PackageExtractArgs,
    },
    /// Package a Rust crate for Debian.
    Package {
        #[command(flatten)]
        init: PackageInitArgs,
        #[command(flatten)]
        extract: PackageExtractArgs,
        #[command(flatten)]
        finish: PackageExecuteArgs,
    },
    /// Package a Rust crate for debcraft (generates debcraft.yaml).
    PackageDebcraft {
        #[command(flatten)]
        init: PackageInitArgs,
        #[command(flatten)]
        extract: PackageExtractArgs,
        #[command(flatten)]
        finish: PackageDebcraftArgs,
    },
    /// Print the transitive dependencies of a package in topological order.
    BuildOrder {
        #[command(flatten)]
        args: BuildOrderArgs,
    },
    /// Print the dependencies of a package in d/control format
    DebDependencies {
        #[command(flatten)]
        args: DebDependenciesArgs,
    },
    /// Update debian/control Build-Depends from workspace Cargo.toml files
    #[cfg(feature = "update-dependencies")]
    UpdateDependencies {
        #[command(flatten)]
        args: UpdateDependenciesArgs,
    },
}
