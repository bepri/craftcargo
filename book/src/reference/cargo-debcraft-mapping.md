Cargo.toml to debcraft.yaml Conversion Analysis
=================================================

This document analyses the feasibility of generating a debcraft.yaml file
directly from a Rust crate's Cargo.toml, using the rules documented in
debcargo-rules.txt as the source of truth for how Cargo metadata maps to
Debian packaging, and debcraft-debian-mapping.txt as the reference for what
debcraft.yaml expects.

The analysis covers:
  A. Fields that map cleanly from Cargo.toml to debcraft.yaml
  B. Fields requiring intermediate translation logic (borrowable from debcargo)
  C. Fields with no Cargo.toml source (must be supplied by the packager)
  D. debcraft.yaml concepts that have no Cargo.toml equivalent
  E. debcargo outputs that have no debcraft.yaml equivalent
  F. Missing features in debcraft required for Rust crate packaging
  G. The dh-cargo build system — detailed flow
  H. Field-by-field mapping table
  I. Summary and recommendations


A. CLEAN MAPPINGS (Cargo.toml -> debcraft.yaml)
================================================

These Cargo.toml fields map directly to debcraft.yaml fields with little or
no transformation.

Cargo.toml field            debcraft.yaml field     Notes
---------------------------------------------------------------------------
[package.name]              name                    Requires transformation:
                                                    underscores -> hyphens,
                                                    lowercase, prefix "rust-".
                                                    See section B.

[package.version]           version                 Requires transformation.
                                                    See section B.

[package.description]       summary / description   Requires splitting and
                                                    prefix-stripping.
                                                    See section B.

[package.license]           license                 Used as-is (SPDX string).
                                                    "/" -> "or",
                                                    " OR " -> "or",
                                                    " AND " -> "and".

[package.homepage] or       source-code             Priority: homepage ->
[package.repository]                                repository -> crates.io URL.
                                                    Note: debcraft source-code
                                                    maps to Vcs-Browser, not
                                                    Homepage. See section E.

[package.authors]           (copyright file)        Used in debian/copyright,
                                                    not in debcraft.yaml itself.
                                                    No debcraft.yaml field.


B. FIELDS REQUIRING TRANSFORMATION (logic borrowable from debcargo)
====================================================================

The following mappings require non-trivial logic that debcargo already
implements and could be reused.

B.1 Package naming
  Cargo.toml [package.name] -> debcraft.yaml name
    Rule: replace '_' with '-', lowercase, prepend "rust-".
    Semver suffix: append "-{MAJOR}" or "-0.{MINOR}" when semver_suffix = true.
    Source: debcargo src/debian/control.rs: base_deb_name(), dsc_name()

B.2 Version translation
  Cargo.toml [package.version] -> debcraft.yaml version
    Rule: format as "{MAJOR}.{MINOR}.{PATCH}", pre-release appended with "~"
    (e.g. 1.0.0-alpha.1 -> 1.0.0~alpha.1), build metadata dropped, repack
    suffix appended with "+".
    Full Debian version: "{upstream_version}-1" (revision starts at 1).
    Source: debcargo src/debian/control.rs: deb_upstream_version()

B.3 Description splitting and cleaning
  Cargo.toml [package.description] -> debcraft.yaml summary and description
    Rules (applied in order):
      1. "\n\n" -> paragraph break; single "\n" -> space (unwrap line wraps).
      2. Strip prefixes: crate name, "This ... is/provides", articles
         (a/an/the), "rust implementation/library/tool/crate of/to/for".
      3. Capitalise first letter.
      4. First sentence or first line -> summary; remainder -> description.
      5. Fallback: summary = Rust crate "{crate_name}" if no description.
    Source: debcargo src/crates.rs: get_summary_description()

    Note: debcargo appends auto-generated suffixes to the summary for each
    package (e.g. " - Rust source code", " - feature \"X\""). These are
    package-specific and need to be generated per-package entry in
    debcraft.yaml.

B.4 Dependency translation
  Cargo.toml [dependencies] -> packages.<name>.depends (and build-packages)
    Rules (full detail in debcargo-rules.txt section 5):
      - Dep name: '_' -> '-', lowercase, prefix "librust-", suffix "-dev".
      - Uses default features: librust-{dep}+default-dev
      - Specific features: librust-{dep}+{feature}-dev
      - No default features: librust-{dep}-dev
      - Version ranges: caret/tilde/exact/range -> Debian (>=)/(<<) clauses,
        with semver-suffixed variant packages (librust-{dep}-1-dev, etc.).
    Source: debcargo src/debian/dependency.rs: deb_dep(), deb_deps()

B.5 Feature package generation
  Cargo.toml [features] -> multiple packages.<name> entries in debcraft.yaml
    Rules (full detail in debcargo-rules.txt section 6):
      - One package per feature: librust-{name}+{feature}-dev
      - Plus one bare library package: librust-{name}-dev
      - Feature packages depend on the bare library.
      - reduce_provides algorithm collapses features with identical deps.
      - Package naming: librust-{base}+{feature}-dev
    Source: debcargo src/debian/mod.rs: prepare_debian_control(),
            reduce_provides(), collapse_features()

B.6 Provides: virtual package versioning
  Crate version -> packages.<name>.provides list
    Rule: generate four versioned virtual packages per library/feature package:
      - librust-{name}-dev (unversioned, omitted if semver_suffix)
      - librust-{name}-{MAJOR}-dev
      - librust-{name}-{MAJOR}.{MINOR}-dev
      - librust-{name}-{MAJOR}.{MINOR}.{PATCH}-dev
    Source: debcargo src/debian/control.rs: Package::new()

B.7 Minimum Rust version -> build-packages toolchain constraint
  Cargo.toml [package.rust-version] -> build-packages rustc entry
    Rule: if rust-version is set, emit "rustc:native (>= {version})";
    otherwise emit "rustc:native".
    Source: debcargo src/debian/mod.rs: toolchain_deps(), rustc_dep()

B.8 Breaks/Replaces for semver-suffix packages
  Crate version + semver_suffix = true -> packages.<name>.breaks / .replaces
    Rule: librust-{base_name}-dev (<< {MAJOR}.{MINOR}.{PATCH+1}~)
    Source: debcargo src/debian/control.rs: Package::new()


C. FIELDS WITH NO CARGO.TOML SOURCE (packager-supplied)
========================================================

These debcraft.yaml fields cannot be derived from Cargo.toml alone. They
must either be defaulted, inferred from conventions, or provided by the
packager in a debcargo.toml-equivalent config file.

debcraft.yaml field     Current debcargo source / default
---------------------------------------------------------------------------
maintainer              debcargo default:
                          "Debian Rust Maintainers
                          <pkg-rust-maintainers@alioth-lists.debian.net>"
                        For Ubuntu/Snapcraft context, a different default
                        would be needed (e.g. Ubuntu Rust Maintainers).

uploaders               From debcargo.toml: uploaders = [...]
                        No Cargo.toml source.

base                    Not in debcargo. Specifies the target Ubuntu release
                        (e.g. "ubuntu@24.04"). Must be provided externally or
                        defaulted to current LTS.

platforms               Not in debcargo. debcargo generates Architecture: any
                        for all packages. debcraft.yaml requires explicit
                        platform declarations or omits them for "all arches".

section (source)        debcargo defaults to "rust" for library crates,
                        "FIXME-IN-THE-SOURCE-SECTION" for binary-only crates.

priority                Not set by debcargo. debcraft defaults to "optional",
                        which is correct for Rust crates.

contact                 No Cargo.toml source. Not set by debcargo.
                        Could be populated from maintainer.

issues                  No Cargo.toml or debcargo source.
                        Could be populated from [package.repository] + "/issues"
                        if the repository is on GitHub/GitLab.

adopt-info              No Cargo.toml or debcargo source. This is a
                        craft-application mechanism for dynamic versioning.
                        Not relevant for Cargo-based packages where version
                        is always known from Cargo.toml.

build-base              No debcargo equivalent. Usually same as base.

package-repositories    No debcargo equivalent. Not normally needed for
                        Rust crates in the Ubuntu archive.


D. debcraft.yaml CONCEPTS WITHOUT CARGO.TOML EQUIVALENT
========================================================

D.1 parts: (build instructions)
  debcraft.yaml requires a `parts:` section describing how to build the
  package. This corresponds to debian/rules in the traditional model.
  debcargo generates `dh $@ --buildsystem cargo` in debian/rules.

  craft-parts (the library underlying debcraft) ships two Rust-related
  plugins, confirmed present in the debcraft venv:

  rust plugin (craft_parts/plugins/rust_plugin.py):
    Builds Rust binary applications. Uses rustup to manage the toolchain
    (or falls back to system rustc/cargo when rust-channel = "none").
    Runs `cargo install` to install binaries into the part install
    directory. Supports rust-features, rust-path, rust-no-default-features,
    rust-use-global-lto, rust-cargo-parameters, rust-inherit-ldflags.
    This plugin is designed for binary targets; it does NOT implement the
    dh-cargo flow for library crate packaging (registering source under
    /usr/share/cargo/registry/, installing .rlib files, etc.).

  cargo-use plugin (craft_parts/plugins/cargo_use_plugin.py):
    Copies a Rust crate's source tree into a backstage cargo registry so
    that other parts can use it as a local dependency (redirects crates.io
    to the local registry via a generated cargo config). It reads Cargo.toml
    to derive the registry directory name ({name}-{version}). It creates
    a minimal .cargo-checksum.json (with empty files map: {"files":{}})
    in the registry entry, which satisfies cargo's checksum requirement
    for local sources but does not carry the crates.io SHA256 hash.
    This plugin is a dependency provider, not a final-package builder.

  For Rust *library* packages (the librust-X-dev pattern): neither plugin
  replicates the dh-cargo build system. A dedicated library-packaging
  plugin or a nil plugin with explicit commands would be needed.
  See section F.1 for details.

D.2 base / build-base
  The target Ubuntu/Debian release. No Cargo.toml equivalent.
  Must be specified by the packager or defaulted.

D.3 platforms
  Architecture declarations. debcargo always generates Architecture: any.
  In debcraft.yaml, this can be omitted to mean "all platforms", or
  specified explicitly. No Cargo.toml source.

D.4 package-repositories
  Additional APT repositories. No Cargo.toml or debcargo equivalent.


E. debcargo OUTPUTS WITHOUT debcraft.yaml EQUIVALENT
=====================================================

The following are generated by debcargo or required for Rust Debian packaging
but have no corresponding field in the current debcraft.yaml schema.

E.1 X-Cargo-Crate and X-Cargo-Crate-Version (CRITICAL)
  debcargo always emits these in the source stanza:
    X-Cargo-Crate: {exact_crate_name}
    X-Cargo-Crate-Version: {plain_upstream_version}
  These are read by dh-cargo to locate the crate in
  /usr/share/cargo/registry/ and construct the correct install path. Without them, dh-cargo
  cannot build the package.
  craftcargo's debcraft.yaml generator does not emit these fields, so this metadata is
  currently dropped in debcraft.yaml output.
  debcraft.yaml has no mechanism for custom source stanza fields.
  This is a BLOCKING gap: debcraft must support X-Cargo-Crate fields or
  provide an equivalent mechanism to tell the cargo build plugin which
  crate name and version to use.

E.2 Build-Depends-Arch vs Build-Depends-Indep distinction
  debcargo splits build dependencies into three fields:
    Build-Depends:        debhelper-compat, dh-sequence-cargo
    Build-Depends-Arch:   toolchain + crate deps (<!nocheck> annotated)
    Build-Depends-Indep:  (when needed)
  debcraft auto-generates Build-Depends from parts.build-packages but does
  not currently distinguish arch/indep categories or support <!nocheck>
  build profile annotations.

E.3 <!nocheck> build profile annotations
  debcargo annotates build dependencies for library-only packages with
  <!nocheck> to allow building without running tests (to break cycles).
  The debcraft parts.build-packages list does not support build profiles.
  This is needed to allow Rust library packages to be bootstrapped without
  circular test dependencies.

E.4 cargo-checksum.json
  debcargo generates debian/cargo-checksum.json with the crates.io SHA256
  checksum. This is required by dh-cargo to verify the orig tarball. There
  is no debcraft.yaml field for this; it would need to be handled by the
  cargo plugin or a helper.

E.5 Vcs-Git field
  debcargo generates both Vcs-Git and Vcs-Browser. debcraft.yaml only has
  source-code, which maps to Vcs-Browser. Vcs-Git would be lost unless
  debcraft adds a vcs-git field or derives it from source-code. craftcargo
  does not emit vcs-git in debcraft.yaml output.

E.6 debian/tests/control (autopkgtest)
  debcargo generates a full debian/tests/control with one test stanza per
  Cargo feature (using /usr/share/cargo/bin/cargo-auto-test). debcraft.yaml
  has no equivalent mechanism for declaring autopkgtests. The entire
  per-feature test matrix (--no-default-features, --features X, flaky
  markers, test_architecture) cannot be expressed in debcraft.yaml.

E.7 debian/watch
  debcargo generates a uscan watch file pointing at crates.io via the
  fakeupstream CGI. debcraft.yaml has no equivalent.

E.8 Rules-Requires-Root
  debcargo supports setting Rules-Requires-Root in debian/control. There
  is no debcraft.yaml field for this.

E.9 Built-Using / Static-Built-Using
  The binary executable package stanza from debcargo includes:
    Built-Using: ${cargo:Built-Using}
    Static-Built-Using: ${cargo:Static-Built-Using}
  These substitution variables are populated by dh-cargo to record which
  Rust crates were statically linked into the binary. debcraft.yaml has no
  support for these fields, and debcraft's shlibdeps helper handles
  shared-library dependencies but not static linking attribution.

E.10 ${cargo:Depends}, ${cargo:Recommends}, ${cargo:Suggests},
     ${cargo:Provides}, ${cargo:Built-Using}, ${cargo:Static-Built-Using}
  debcargo uses cargo-specific dh-cargo substitution variables in the binary
  executable package stanza. debcraft.yaml explicitly does not support
  substitution variables. The cargo plugin would need to handle these
  internally and inject the resolved values.

E.11 debian/source/format
  debcargo always generates "3.0 (quilt)". debcraft reads this file but it
  is not a debcraft.yaml field; it must exist on disk.

E.12 Lintian overrides per feature package
  debcargo generates a {pkg}.lintian-overrides file for each feature package
  suppressing "empty-rust-library-declares-provides". debcraft supports
  lintian-overrides files via debian/<pkg>.lintian-overrides. This maps over
  but would need to be auto-generated for each feature package.


F. MISSING FEATURES IN debcraft FOR RUST CRATE PACKAGING
=========================================================

The following features would need to be added to or clarified in debcraft
to support generating debcraft.yaml from Cargo.toml.

F.1 No library-packaging plugin for dh-cargo behaviour (CRITICAL)
  craft-parts ships a "rust" plugin and a "cargo-use" plugin (confirmed
  present in the debcraft venv). However, neither replicates the dh-cargo
  build system that debcargo relies on.

  The "rust" plugin runs `cargo install` to produce installable binaries.
  It is suitable for packaging binary crates but does NOT:
  - Install crate source into /usr/share/cargo/registry/{name}-{version}/
  - Install compiled .rlib files for other packages to link against
  - Handle the per-feature test loop (cargo test --no-default-features,
    cargo test --features X)
  - Set up the X-Cargo-Crate / X-Cargo-Crate-Version registry lookup

  The "cargo-use" plugin copies a crate into a local backstage registry
  for use as a build-time dependency, not for producing the final installable
  package.

  For Rust *library* crate packaging (the librust-X-dev model), a dedicated
  craft-parts plugin equivalent to dh-cargo is still needed, handling:
  - Installing source tree to /usr/share/cargo/registry/{name}-{version}/
  - Generating or consuming a real cargo-checksum.json with crates.io hash
  - Running per-feature test suites (--no-default-features, --features X)
  - Respecting X-Cargo-Crate and X-Cargo-Crate-Version for registry lookup

  For binary crate packaging the existing "rust" plugin can be used directly
  with plugin: rust and rust-channel: none (to use the system toolchain).

F.2 Custom source stanza fields (X-Cargo-Crate, X-Cargo-Crate-Version)
  debcraft.yaml needs a mechanism to include custom fields in the generated
  source stanza (dpkg source control file). At minimum, X-Cargo-Crate and
  X-Cargo-Crate-Version must be emittable.

  Possible approach: a top-level extra-fields or custom-fields map in
  debcraft.yaml, similar to debcargo's extra_lines for binary packages.

F.3 Build profile support in build-packages (<!nocheck>)
  To break circular build-time test dependencies among Rust library packages,
  build-packages entries need to support the <!nocheck> build profile
  annotation. This is standard dpkg behaviour but debcraft's parts
  build-packages list does not currently support profile qualifiers.

  Possible approach: allow build-packages entries to be objects with optional
  profile: field, or support raw dpkg dependency strings including profiles.

F.4 Build-Depends-Arch and Build-Depends-Indep fields
  debcraft generates Build-Depends from parts.build-packages but does not
  expose Build-Depends-Arch or Build-Depends-Indep as separate fields.
  For Rust packages, the split is important:
  - Build-Depends: debhelper-compat, dh-sequence-cargo (always needed)
  - Build-Depends-Arch: toolchain + crate deps (architecture-specific)
  This affects cross-compilation correctness.

  Possible approach: craft-grammar architecture conditionals can approximate
  Build-Depends-Arch if they work correctly with dpkg source generation.

F.5 Autopkgtest / debian/tests/control support
  debcraft has no mechanism to declare autopkgtests. For Rust packages
  this is significant: debcargo generates a full per-feature test matrix
  using cargo-auto-test. Without autopkgtest support, automated post-install
  testing of Rust crates would be lost.

  Possible approach: a new top-level tests: section in debcraft.yaml that
  maps to debian/tests/control, at minimum supporting:
  - test-command
  - depends
  - restrictions (allow-stderr, skip-not-installable, flaky)
  - architecture

F.6 Multi-package Provides: with version-locked entries
  debcargo generates Provides: entries of the form:
    librust-foo-dev (= ${binary:Version})
  debcraft.yaml's packages.<name>.provides list accepts strings. The
  "${binary:Version}" substitution variable is explicitly not supported.
  debcraft would need to either:
  - Support a limited set of substitution variables in provides/depends, or
  - Generate the correct versioned provides at build time from the package
    version automatically.

F.7 Vcs-Git field
  debcraft.yaml has source-code which maps to Vcs-Browser. A separate
  vcs-git field (or a more general vcs: section with browser/git sub-keys)
  would be needed to carry both VCS fields that Rust team packages require.

F.8 Built-Using and Static-Built-Using fields
  The cargo plugin needs to populate Built-Using and Static-Built-Using in
  generated binary control files. This requires post-build introspection of
  which Rust crates were statically compiled in. No current debcraft mechanism
  handles this.

F.9 cargo-checksum.json generation
  The cargo plugin must generate or supply debian/cargo-checksum.json
  containing the crates.io SHA256 hash of the source archive. This is
  currently produced by debcargo from crates.io registry metadata. The
  debcraft cargo plugin would need access to this checksum.

F.10 Per-feature lintian-override auto-generation
  debcargo auto-generates {pkg}.lintian-overrides for every feature package
  suppressing the "empty-rust-library-declares-provides" tag. debcraft
  supports lintian-overrides files in debian/ but does not auto-generate them.
  The conversion tool would need to emit these files into debcraft/ alongside
  debcraft.yaml, or debcraft's lintian helper would need to suppress this tag
  automatically for Rust packages.

F.11 Rules-Requires-Root field
  debcraft.yaml has no field for Rules-Requires-Root. This is rarely needed
  for Rust packages, but completeness requires a mechanism for it (e.g., a
  top-level extra-source-fields map).


G. THE dh-cargo BUILD SYSTEM — DETAILED FLOW
=============================================

This section documents the dh-cargo build system (v33, current in Debian
unstable) in full detail. Source: https://salsa.debian.org/rust-team/dh-cargo

The system consists of four installed files:

  /usr/share/perl5/Debian/Debhelper/Buildsystem/cargo.pm
      Main debhelper build system module. Implements configure, build, test,
      install, and clean phases.

  /usr/share/perl5/Debian/Debhelper/Sequence/cargo.pm
      Sequence addon (provides dh-sequence-cargo). Adds
      -XCargo.toml.orig to dh_clean so normalised manifests are preserved.

  /usr/share/cargo/bin/cargo-auto-test
      DEP-8 autopkgtest runner script for per-feature testing.

  /usr/share/cargo/bin/dh-cargo-built-using
      Generates cargo:Built-Using and cargo:Static-Built-Using substvars
      by inspecting build artefacts.

A companion Python wrapper at /usr/share/cargo/bin/cargo (from the Debian
cargo package, not dh-cargo) intercepts all cargo invocations and injects
Debian-specific flags. It only activates when CARGO_HOME ends in
"debian/cargo_home"; otherwise it passes through to /usr/bin/cargo unchanged.


G.1 Pre-build step (runs before every phase)
--------------------------------------------
cargo.pm sub pre_building_step:

  1. Fix timestamps for reproducibility:
       find . ! -newermt 'jan 01, 2000' -exec touch -d@$SOURCE_DATE_EPOCH {} +

  2. Determine crate name and version:
       X-Cargo-Crate from debian/control source stanza; fallback: strip
       "rust-" prefix and semver suffix (e.g. "-1") from Source: field.
       X-Cargo-Crate-Version from debian/control; fallback: upstream part
       of the version in debian/changelog.

  3. Set environment variables:
       CARGO_HOME         = <abs-path>/debian/cargo_home
       DEB_CARGO_CRATE    = {crate}_{version}
       DEB_HOST_RUST_TYPE = <target triple from rustc --version --verbose>
       DEB_HOST_GNU_TYPE  = <from dpkg-architecture>

  The CARGO_HOME value ending in "debian/cargo_home" is the trigger that
  activates all Debian-specific behaviour in the /usr/share/cargo/bin/cargo
  Python wrapper.


G.2 Configure phase
--------------------
cargo.pm sub configure:

  cp debian/cargo-checksum.json  .cargo-checksum.json
  rm -f Cargo.lock
  /usr/share/cargo/bin/cargo  prepare-debian  debian/cargo_registry  \
      --link-from-system

What "cargo prepare-debian" does (Python wrapper):
  - Creates debian/cargo_registry/ directory.
  - Symlinks every entry from /usr/share/cargo/registry/ into
    debian/cargo_registry/ (making all installed librust-*-dev packages
    available as local dependencies without network access).
  - Writes debian/cargo_home/config:

      [source.crates-io]
      replace-with = "dh-cargo-registry"

      [source.dh-cargo-registry]
      directory = "<abs>/debian/cargo_registry"

      [build]
      rustflags = [
        "-C", "debuginfo=2",
        "--cap-lints", "warn",
        "-C", "linker=<DEB_HOST_GNU_TYPE>-gcc",
        "-C", "link-arg=...",         (LDFLAGS entries)
        "--remap-path-prefix", "<srcdir>=<registry>/<crate>",
        "--remap-path-prefix", "<registry-path>=/usr/share/cargo/registry"
      ]

  This config redirects all crates.io lookups to the local symlink
  directory; no network access ever occurs during the build.


G.3 Build phase
----------------
cargo.pm sub build:

For LIBRARY packages (librust-*-dev): no cargo invocation.
  - Copies the entire source tree (excluding .git, .pc, debian/, target/)
    into debian/.debhelper/_source/copied_sources/ for use in the install
    step.
  - Copies debian/cargo-checksum.json into copied_sources/ as
    .cargo-checksum.json.
  - Applies any -X exclude patterns (EXCLUDE_FIND) via find ... -delete.

For BINARY EXECUTABLE packages:
  - The build step is skipped; compilation happens during the install step
    via "cargo install".


G.4 Test phase
---------------
cargo.pm sub test:

  /usr/share/cargo/bin/cargo  {build|test}  [extra args passed via dh]

Standard debian/rules from debcargo:
  override_dh_auto_test:
      dh_auto_test -- test --all

  This becomes:
    /usr/share/cargo/bin/cargo test --all [extra flags]

  If only compile-checking is wanted (no --all), the wrapper runs:
    /usr/share/cargo/bin/cargo build

The Python wrapper prepends to every subcommand:
  -Zavoid-dev-deps --verbose --verbose -j{N} --target {DEB_HOST_RUST_TYPE}
where N comes from DEB_BUILD_OPTIONS=parallel=N (default: nproc).

After the test invocation, the test phase also runs:
  env CARGO_CHANNEL=debug  /usr/share/cargo/bin/dh-cargo-built-using
to generate cargo:Built-Using / cargo:Static-Built-Using substvars for
debug build artefacts (see G.8).


G.5 Install phase
------------------
cargo.pm sub install:

For the BASE LIBRARY package (librust-{name}-dev):
  Target dir: debian/{libpkg}/usr/share/cargo/registry/{crate}-{version}/
  (tilde characters in version are replaced with hyphens)

  install -d  <target>
  cp --parents -at <target>  <all files from copied_sources>
  find <target> -name ... -delete    (apply -X excludes)
  touch -d@$SOURCE_DATE_EPOCH  <target>/Cargo.toml

For each FEATURE package (librust-{name}+{feat}-dev):
  Only a doc symlink is installed:
  ln -s {libpkg}  debian/{featurepkg}/usr/share/doc/{featurepkg}

  Feature packages contain NO source files of their own. They are pure
  dependency pivot points: installing them pulls in the base library plus
  the feature's additional crate dependencies.

For BINARY EXECUTABLE packages:
  env DESTDIR={tmpdir}  /usr/share/cargo/bin/cargo  install  [args]
  /usr/share/cargo/bin/dh-cargo-built-using  {binpkg}

  The wrapper runs:
    cargo install --path . --root $DESTDIR/usr [--features ...] [--locked]
  followed by removing .crates.toml and .crates2.json metadata files from
  the install root.


G.6 Clean phase
----------------
cargo.pm sub clean:

  touch --no-create -d@$SOURCE_DATE_EPOCH  .cargo_vcs_info.json
  /usr/share/cargo/bin/cargo  clean
  rm -f  .cargo-checksum.json
  rm -rf debian/cargo_registry


G.7 cargo-checksum.json
------------------------
Format (exact, as generated by debcargo):

  {"package":"<sha256hex>","files":{}}

  "package": SHA-256 hash of the .crate tarball downloaded from crates.io,
             obtained by debcargo from the crates.io registry API at
             package-generation time.
  "files":   empty object {} (cargo normally stores per-file hashes but
             accepts the empty form for directory sources).

This file must be present in debian/ before the build starts. It is:
  - Copied to .cargo-checksum.json in the configure step (for the build
    to verify the source directory).
  - Copied into copied_sources/ in the build step.
  - Installed into /usr/share/cargo/registry/{crate}-{version}/ so that
    packages that depend on this crate can verify it during their builds.

Without this file the configure step fails immediately.

Source: debcargo src/debian/mod.rs:344-353


G.8 Built-Using and Static-Built-Using generation
---------------------------------------------------
/usr/share/cargo/bin/dh-cargo-built-using:

  1. Reads .d dependency files from:
       target/{DEB_HOST_RUST_TYPE}/{CARGO_CHANNEL}/deps/*.d
     (CARGO_CHANNEL is "debug" during test phase, "release" during install)

  2. For each path in .d files that is under debian/cargo_registry/:
     - Resolves the symlink back to the system registry path
     - Runs: dpkg -S <path>  to find the installed Debian binary package
     - Runs: dpkg -p <pkg>  to get the source package name and version

  3. cargo:Built-Using: includes only source packages whose copyright
     file contains a copyleft license (GPL, LGPL, AGPL, GFDL, MPL, CDDL,
     CPL, Artistic, Perl, QPL). Format: "src (= version), ..."

  4. cargo:Static-Built-Using: includes ALL statically-linked source
     packages regardless of license. Same format.

  5. Writes both to debian/{pkg}.substvars:
       cargo:Built-Using=src1 (= ver1), src2 (= ver2)
       cargo:Static-Built-Using=src1 (= ver1), src2 (= ver2)

Note: cargo:Depends, cargo:Provides, cargo:Recommends, cargo:Suggests are
NOT populated by any dh-cargo script. They expand to empty strings in the
generated control file. They are placeholder hooks for potential future use.


G.9 dh-sequence-cargo sequence addon
--------------------------------------
/usr/share/perl5/Debian/Debhelper/Sequence/cargo.pm:

  add_command_options('dh_clean', '-XCargo.toml.orig');

Effect: dh_clean does not remove Cargo.toml.orig files (created when
debcargo normalises Cargo.toml via "cargo package").

The dh-cargo package provides "dh-sequence-cargo" so that declaring
  Build-Depends: dh-sequence-cargo
in debian/control activates the cargo buildsystem automatically without
needing --with cargo in debian/rules. debcargo always emits this dependency.


G.10 Autopkgtest — cargo-auto-test
------------------------------------
/usr/share/cargo/bin/cargo-auto-test <crate> <version> [extra flags]

Full DEP-8 test flow:

  1. cd /usr/share/cargo/registry/${crate}-${version}
  2. Create rundir=$(mktemp -d)
  3. Set environment:
       CARGO_HOME="$rundir/debian/cargo_home"
       CARGO_TARGET_DIR="$rundir/target"
       PATH="/usr/share/cargo/bin:$PATH"
       DEB_CARGO_CRATE="${crate}_${version}"
       DEB_HOST_RUST_TYPE=<from rustc --version --verbose>
     Plus: dpkg-buildflags --export, dpkg-architecture -s
  4. Register installed crates:
       cargo prepare-debian "$rundir/registry" --link-from-system
  5. Run tests:
       cargo test [--color=always] "$@"
  6. Error handling:
       On E0554 (unstable feature used): retry with RUSTC_BOOTSTRAP=1
       On E0463+E0465 (missing crate): silently ignore (cargo bug #6819)
  7. Cleanup: rm -rf "$rundir"

Per-feature test stanzas in debian/tests/control (generated by debcargo):

  Test-Command: /usr/share/cargo/bin/cargo-auto-test {crate} {version}
                --all-targets [--no-default-features] [--features {feat}]
  Features: test-name={pkg}:{feature}
  Depends: dh-cargo (>= 33~), @
  Restrictions: allow-stderr, skip-not-installable[, flaky]

  debcargo generates one stanza per feature plus one with --all-features.
  The "flaky" restriction is added when the crate is marked as such in
  debcargo.toml (skip_tests = true or test_is_broken = true).


G.11 Environment variables summary
------------------------------------

Variable               Set by                  Purpose
-------------------------------------------------------------------------
CARGO_HOME             cargo.pm                Always debian/cargo_home (abs).
                                               Activates Python wrapper.
DEB_CARGO_CRATE        cargo.pm                {crate}_{version}
DEB_HOST_RUST_TYPE     cargo.pm                Target triple, e.g.
                                               x86_64-unknown-linux-gnu
DEB_HOST_GNU_TYPE      cargo.pm                Linker prefix, e.g.
                                               x86_64-linux-gnu
CARGO_TARGET_DIR       cargo-auto-test         $rundir/target (test only)
CARGO_CHANNEL          dh-cargo test/install   "debug" (test) / "release"
                                               (install); used by
                                               dh-cargo-built-using
CFLAGS/CXXFLAGS/       dpkg-buildflags         Injected into rustflags via
CPPFLAGS/LDFLAGS                               cargo config by Python wrapper
RUSTFLAGS              caller                  Moved into cargo_home/config
                                               (workaround for cargo #6338)
DEB_BUILD_OPTIONS      dpkg                    parallel=N -> -jN;
                                               nodoc skips docs;
                                               nocheck skips tests
DEB_BUILD_PROFILES     dpkg                    nodoc, nocheck recognised
DESTDIR                dh / cargo.pm           Install destination for bins
DEB_CARGO_INSTALL_     optional                Binary install prefix
  PREFIX                                       (default /usr)
DEB_CARGO_CRATE_IN_    optional                1 = use registry; 0 = --path .
  REGISTRY
CARGO_REGISTRY         optional                Override registry path
                                               (default debian/cargo_registry)
SOURCE_DATE_EPOCH      dpkg                    Timestamp reproducibility
RUSTC_BOOTSTRAP        cargo-auto-test         Set to 1 on E0554 retry


G.12 /usr/share/cargo/registry/ structure
-------------------------------------------
Each installed librust-*-dev package contributes one directory:

  /usr/share/cargo/registry/
      {crate_name}-{version}/
          Cargo.toml              (timestamp fixed to SOURCE_DATE_EPOCH)
          .cargo-checksum.json    ({"package":"<sha256>","files":{}})
          src/
          ...                     (full crate source tree, no target/)

Directory naming: tilde in version replaced by hyphen
  e.g. librust-foo-0.1.0~alpha.1-dev installs to foo-0.1.0-alpha.1/

The configure step creates debian/cargo_registry/ as a flat symlink farm
pointing into /usr/share/cargo/registry/, and sets up cargo_home/config
to redirect crates.io lookups to debian/cargo_registry/. Builds are fully
offline: all dependencies must be pre-installed as Debian packages.


G.13 Offline build model vs vendor directory
----------------------------------------------
dh-cargo uses the SYSTEM REGISTRY model, not a vendor directory:
  - All crate dependencies must be installed as librust-*-dev packages.
  - The configure step symlinks the system registry into debian/cargo_registry/.
  - No network access occurs during dpkg-buildpackage.

The VENDOR DIRECTORY model (cargo vendor) is used only when packaging the
cargo binary itself (and similarly self-hosting tools), where the Debian
archive cannot yet supply all dependencies. It is NOT used for ordinary
Rust crate packages in the Debian/Ubuntu archive.


H. FIELD-BY-FIELD MAPPING TABLE
=================================

The following table summarises every debcraft.yaml field and its likely
source when generating from Cargo.toml.

Legend:
  AUTO  = can be derived automatically from Cargo.toml / debcargo rules
  CFG   = must come from a debcargo.toml-style config file
  FIXED = fixed value / convention
  N/A   = not applicable / not needed for Rust crates
  MISS  = requires a missing debcraft feature (see section F)

debcraft.yaml field             Source / status
---------------------------------------------------------------------------
TOP-LEVEL FIELDS:
name                            AUTO  [package.name] + naming rules
version                         AUTO  [package.version] + version rules +
                                      "-1" debian revision
summary                         AUTO  [package.description] + stripping rules
description                     AUTO  [package.description] remainder
maintainer                      CFG   debcargo.toml maintainer; default:
                                      Ubuntu Rust Maintainers (new default
                                      needed vs Debian default)
original-maintainer             N/A   not used for new Ubuntu Rust packages
uploaders                       CFG   debcargo.toml uploaders
section                         AUTO  "rust" for library; CFG for binary-only
priority                        FIXED "optional"
contact                         CFG   can default to maintainer
source-code                     AUTO  [package.homepage] or [package.repository]
                                      or crates.io URL (note: maps to
                                      Vcs-Browser; Vcs-Git is MISS F.7)
license                         AUTO  [package.license] (SPDX), normalised
issues                          AUTO  could derive from [package.repository]
                                      + "/issues" for GitHub/GitLab repos
adopt-info                      N/A
base                            CFG   packager-specified target Ubuntu release
build-base                      CFG   usually same as base
platforms                       FIXED omit (means "all architectures") or
                                      CFG
package-repositories            N/A

parts:                          PARTIAL craft-parts "rust" plugin exists for
  {crate_name}:                         binary targets; library packaging
    plugin: rust                        needs a dh-cargo equivalent plugin
    rust-channel: none          FIXED   use system toolchain                (F.1)
    source: .                   FIXED
    build-packages:             AUTO  toolchain + translated [dependencies]
                                      (missing <!nocheck> support: F.3)
                                      (missing Build-Depends-Arch: F.4)

PACKAGE-LEVEL FIELDS (one entry per generated Debian binary package):
packages.<name> (key)           AUTO  librust-{name}-dev for bare library,
                                      librust-{name}+{feat}-dev per feature,
                                      {bin_name} for binary executable
packages.<name>.architectures   FIXED "any" for all Rust packages
packages.<name>.summary         AUTO  [package.description] + per-pkg suffix
packages.<name>.description     AUTO  per-package generated text
packages.<name>.version         N/A   inherits project version
packages.<name>.depends         AUTO  translated Cargo dependencies
                                      (${misc:Depends} dropped, MISS F.6
                                      for ${binary:Version} in Provides)
packages.<name>.recommends      AUTO  default feature + super-features
                                      (bare lib package only)
packages.<name>.suggests        AUTO  non-default features
                                      (bare lib package only)
packages.<name>.provides        AUTO  versioned virtual packages
                                      (MISS F.6 for ${binary:Version})
packages.<name>.breaks          AUTO  semver-suffix packages only
packages.<name>.replaces        AUTO  semver-suffix packages only
packages.<name>.conflicts       CFG   manual override only
packages.<name>.section         AUTO  "rust" for lib; CFG for bin packages
packages.<name>.multi-arch      FIXED "same" for all library packages;
                                      "no" (omitted) for binary packages

NOT IN debcraft.yaml (missing or unrepresentable):
X-Cargo-Crate                   MISS  F.2
X-Cargo-Crate-Version           MISS  F.2
<!nocheck> build profiles       MISS  F.3
Build-Depends-Arch/Indep split  MISS  F.4
debian/tests/control            MISS  F.5
Built-Using                     MISS  F.8
Static-Built-Using              MISS  F.8
cargo-checksum.json             MISS  F.9
debian/watch                    N/A   no debcraft equivalent; separate concern
Vcs-Git                         MISS  F.7
Rules-Requires-Root             MISS  F.11


I. SUMMARY AND RECOMMENDATIONS
================================

Feasibility:
  A basic debcraft.yaml for a simple Rust library crate (no binary targets,
  standard features, no platform-specific deps) can be largely auto-generated
  from Cargo.toml using the transformation rules from debcargo. The metadata
  fields (name, version, summary, description, license, depends, provides,
  breaks, replaces, multi-arch) all have clear derivation rules.

  However, a Rust package cannot be built using the generated debcraft.yaml
  until a cargo craft-parts plugin exists (F.1). This is the single most
  critical missing piece.

Blocking issues (package will not build without these):
  F.1  cargo craft-parts plugin
  F.2  X-Cargo-Crate / X-Cargo-Crate-Version source stanza fields
  F.9  cargo-checksum.json generation

High-priority issues (significant functionality loss):
  F.3  <!nocheck> build profiles (breaks bootstrapping of lib packages)
  F.4  Build-Depends-Arch/Indep split (cross-compilation correctness)
  F.5  Autopkgtest support (per-feature test matrix)
  F.6  ${binary:Version} in Provides (versioned virtual packages)
  F.8  Built-Using / Static-Built-Using (FOSS compliance, reproducibility)

Lower-priority issues (completeness/policy compliance):
  F.7  Vcs-Git field
  F.10 Per-feature lintian-override auto-generation
  F.11 Rules-Requires-Root field

Recommended approach:
  1. For binary crates: use the existing craft-parts "rust" plugin
     (plugin: rust, rust-channel: none to use the system toolchain).
  2. For library crates: implement a dh-cargo-equivalent craft-parts plugin
     that installs source to /usr/share/cargo/registry/ and handles the
     per-feature test loop.
  3. Add X-Cargo-Crate and X-Cargo-Crate-Version as first-class fields or
     via an extra-source-fields escape hatch in debcraft.yaml.
  4. Add <!nocheck> build profile support to parts.build-packages.
  5. Implement a conversion tool (analogous to debcargo) that reads Cargo.toml
     (and an optional debcargo.toml-style config) and outputs debcraft.yaml
     with all binary package entries pre-populated.
  6. Add autopkgtest support to debcraft.yaml as a future enhancement.
  7. Resolve ${binary:Version} by having the cargo plugin auto-generate
     versioned Provides at build time.
