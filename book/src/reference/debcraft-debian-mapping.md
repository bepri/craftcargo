debcraft.yaml Field Origin Mapping from debian/ Directory
==========================================================

This document describes how each field in debcraft.yaml is populated when
converting or mapping from a traditional Debian source package's debian/
directory. It is intended as a reference for code generation.

Overview
--------

debcraft.yaml is the debcraft equivalent of the traditional debian/ packaging
directory. It combines the metadata from debian/control, debian/changelog, and
debian/copyright into a single structured YAML file, and replaces the
procedural debian/rules build system with declarative `parts` (from
craft-parts).

The debcraft.yaml schema is defined in schema/debcraft.json. The models are
implemented in debcraft/models/project.py and debcraft/models/package.py.

When building, the gencontrol helper (debcraft/helpers/gencontrol.py) reads
the debcraft.yaml and generates the binary package control file. Other helpers
handle installation of changelogs, copyright files, lintian overrides, etc.

Additionally, a debcraft/ directory in the source tree can exist alongside the
debian/ directory. Files in debcraft/ take precedence over files in debian/
for all package-specific data files (install lists, maintainer scripts, etc.).


TOP-LEVEL debcraft.yaml FIELDS
================================

name
----
  Type: string (required)
  Constraints: >= 2 chars, matches /^[a-z0-9][a-z0-9.+-]+$/

  Origin: debian/control, `Source:` field in the source package stanza.

  The source package name in Debian. Must follow Debian package naming rules:
  only lowercase letters (a-z), digits (0-9), plus (+), minus (-), and periods
  (.), at least two characters long, starting with an alphanumeric character.

  Example debian/control:
    Source: libpng1.6

  Example debcraft.yaml:
    name: libpng1.6


version
-------
  Type: string | null (optional, max 32 chars)

  Origin: debian/changelog, first (topmost) entry's version field.

  The debian/changelog format is:
    <source-package-name> (<version>) <distribution>; urgency=<urgency>

  The version string is the value between parentheses in the first changelog
  entry. For non-native packages it takes the form <upstream-version>-<debian-revision>
  (e.g., "1.6.43-5build1"). For native packages (where source and binary are
  the same) there is no debian revision (e.g., "2.0").

  The package format (native vs. non-native) is determined by
  debian/source/format. A version containing a hyphen (-) is considered
  non-native; one without is considered native. debcraft validates that the
  version and debian/source/format are consistent at build time (see
  debcraft/services/lifecycle.py:_is_native_package()).

  Example debian/changelog first line:
    libpng1.6 (1.6.43-5build1) noble; urgency=medium

  Example debcraft.yaml:
    version: 1.6.43-5build1

  Note: version can also be set per package under packages.<name>.version.
  When a package has no version set, it inherits the project-level version.


summary
-------
  Type: string | null (optional, max 78 chars)

  Origin: debian/control, the short description (first line of `Description:`)
  in the source package stanza. In Debian policy, the source stanza rarely has
  a Description field; summary may instead come from the first binary package
  stanza's Description short line, or be set manually.

  In debcraft, summary serves as the default short description for binary
  packages that do not define their own summary. At build time (gencontrol.py),
  the effective description for a binary package is:
    summary + "\n" + description
  which maps to the Debian control Description field format:
    Description: <summary>
     <description (extended)>

  Example debian/control Description:
    Description: PNG library - runtime (version 1.6)
     libpng is a library implementing an interface ...

  Here "PNG library - runtime (version 1.6)" is the summary.

  Example debcraft.yaml:
    summary: PNG library - runtime (version 1.6)


description
-----------
  Type: string | null (optional)

  Origin: debian/control, the extended description (all lines after the first
  line of `Description:`) in the source or a binary package stanza.

  In Debian control files, the extended description lines are prefixed with a
  single space. Blank lines in the extended description are represented as
  " ." (a space followed by a period). In debcraft.yaml the description is a
  plain multi-line YAML string (using the `|` block scalar); blank lines are
  represented as empty lines.

  Like summary, description serves as the project-level default for binary
  packages that do not define their own description.

  Example debian/control:
    Description: PNG library - runtime (version 1.6)
     libpng is a library implementing an interface for reading and writing
     PNG (Portable Network Graphics) format files.
     .
     This package contains the runtime library files needed to run software
     using libpng.

  Example debcraft.yaml:
    description: |
      libpng is a library implementing an interface for reading and writing
      PNG (Portable Network Graphics) format files.

      This package contains the runtime library files needed to run software
      using libpng.


maintainer
----------
  Type: string (required)

  Origin: debian/control, `Maintainer:` field in the source package stanza.

  The maintainer's name and email address in RFC 5322 format:
    Name <email@example.com>

  This field is required in debcraft.yaml, mirroring the required nature of
  Maintainer: in debian/control. At build time, the maintainer is written to
  every generated binary package control file.

  Example debian/control:
    Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>

  Example debcraft.yaml:
    maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>


original-maintainer
-------------------
  Type: string | null (optional)

  Origin: debian/control, `XSBC-Original-Maintainer:` or
  `XS-Original-Maintainer:` field in the source package stanza.

  Used in Ubuntu packages when the Ubuntu maintainer differs from the original
  Debian maintainer. The field identifies who maintains the package in Debian.
  At build time, this is written as `Original-Maintainer:` in the binary
  package control files.

  Example debian/control:
    XSBC-Original-Maintainer: Maintainers of libpng1.6 packages <libpng1.6@packages.debian.org>

  Example debcraft.yaml:
    original-maintainer: Maintainers of libpng1.6 packages <libpng1.6@packages.debian.org>


uploaders
---------
  Type: list[string] | null (optional)

  Origin: debian/control, `Uploaders:` field in the source package stanza.

  A comma-separated list of co-maintainers in Debian control format. In
  debcraft.yaml it is expressed as a YAML list of strings. Each string is in
  RFC 5322 Name <email> format.

  Example debian/control:
    Uploaders: Alice Developer <alice@example.com>, Bob Developer <bob@example.com>

  Example debcraft.yaml:
    uploaders:
      - Alice Developer <alice@example.com>
      - Bob Developer <bob@example.com>


section
-------
  Type: string | null (optional)

  Origin: debian/control, `Section:` field in the source package stanza.

  Specifies the archive section (e.g., libs, utils, devel, net, etc.) for the
  source package. This value is used as the default section for all binary
  packages that do not define their own section. At build time (gencontrol.py),
  if neither the package nor the project section is set, an error is raised.

  Example debian/control:
    Section: libs

  Example debcraft.yaml:
    section: libs


priority
--------
  Type: enum (optional, default: "optional")
  Values: "required" | "important" | "standard" | "optional"

  Origin: debian/control, `Priority:` field in the source package stanza.

  Indicates the importance of the package for the system. "extra" (deprecated
  in Debian policy) is not supported; use "optional" instead. Defaults to
  "optional" which is the most common value for user-space packages.

  Example debian/control:
    Priority: optional

  Example debcraft.yaml:
    priority: optional


contact
-------
  Type: string | list[string] | null (optional)

  Origin: debian/control, `Maintainer:` field (or a separate contact address).

  In debcraft.yaml this is inherited from craft-application and can hold a
  contact address for the project. It is distinct from `maintainer`, which is
  the Debian package maintainer. There is no direct 1:1 field in debian/control
  for this field beyond Maintainer:. May be populated from Maintainer: when
  no separate contact address is available.

  Code comment in project.py: "contact -> Maintainer"


source-code
-----------
  Type: string (URI) | null (optional)

  Origin: debian/control, `Vcs-Browser:` field in the source package stanza.

  The URL of the version control system browser (web interface) for the source
  code. In debian/control this is typically paired with a Vcs-* field
  (e.g., Vcs-Git:).

  Code comment in project.py: "source-code -> Vcs-Browser"

  Example debian/control:
    Vcs-Browser: https://salsa.debian.org/debian/libpng1.6
    Vcs-Git: https://salsa.debian.org/debian/libpng1.6.git

  Example debcraft.yaml:
    source-code: https://salsa.debian.org/debian/libpng1.6


license
-------
  Type: string | null (optional)

  Origin: debian/copyright (DEP-5 format), `License:` field, typically the
  SPDX identifier for the primary license (e.g., "GPL-2.0-or-later",
  "Apache-2.0", "MIT").

  debian/copyright uses the Machine-Readable Debian Copyright Format (DEP-5).
  The license identifier is found in the `License:` field of the Files or
  License paragraphs.

  Example debian/copyright:
    License: LGPL-2.0-or-later

  Example debcraft.yaml:
    license: LGPL-2.0-or-later


issues
------
  Type: string | list[string] | null (optional)

  Origin: No direct debian/control equivalent. May be populated from a bug
  tracker URL if one is present in the package metadata or upstream metadata.
  This is a craft-application field with no Debian counterpart.


adopt-info
----------
  Type: string | null (optional)

  Origin: No debian/control equivalent. This is a craft-application field that
  refers to the name of a `parts` entry from which version information is
  adopted. Must match a key in the `parts` map. Used to pull version metadata
  dynamically from the build.


base
----
  Type: string | null (optional)

  Origin: No direct debian/control equivalent. Specifies the Ubuntu base
  (e.g., "ubuntu@24.04") used as the build and runtime environment. This
  determines which Ubuntu release the package targets. In traditional Debian
  packaging, the target distribution is specified in debian/changelog
  (e.g., "noble", "jammy") but is not stored in debian/control directly.

  Example debian/changelog first line (distribution field):
    libpng1.6 (1.6.43-5build1) noble; urgency=medium

  The distribution codename "noble" maps to Ubuntu 24.04, which corresponds to
  base: ubuntu@24.04 in debcraft.yaml.


build-base
----------
  Type: string | null (optional)

  Origin: No direct debian/control equivalent. Specifies the build environment
  base separately from the runtime base. In traditional Debian packaging, the
  build environment is implicitly the same as the runtime environment.


platforms
---------
  Type: dict | null (optional)

  Origin: debian/control, `Architecture:` fields across all binary package
  stanzas. The set of architectures listed in binary package stanzas determines
  which platforms the source package builds for.

  In debcraft.yaml, platforms is a dictionary mapping platform names (usually
  Debian architecture names like "amd64", "arm64") to build-on/build-for pairs.
  If not set, debcraft defaults to building for all supported architectures
  without cross-compilation (see services/project.py:_app_render_legacy_platforms()).

  When the platform name is a valid Debian architecture (one of: amd64, arm64,
  armhf, i386, ppc64el, riscv64, s390x), the build-on and build-for sub-keys
  can be omitted and default to that architecture.

  Supported Debian architectures: amd64, arm64, armhf, i386, ppc64el, riscv64,
  s390x.

  Example debian/control (Architecture fields across binary stanzas):
    Architecture: any       <- architecture-specific (any arch)
    Architecture: all       <- architecture-independent

  Example debcraft.yaml:
    platforms:
      amd64:
        build-on: [amd64]
        build-for: [amd64]
      arm64:
        build-on: [amd64, arm64]
        build-for: [arm64]


parts
-----
  Type: dict (required)

  Origin: debian/rules and debian/patches/.

  This has no direct 1:1 mapping in debian/control. In traditional Debian
  packaging, debian/rules is a Makefile that defines how to build the package.
  debcraft.yaml replaces debian/rules with a declarative `parts` map using
  craft-parts plugins (e.g., "autotools", "cmake", "nil", etc.).

  Each part defines:
  - plugin: The build system (autotools, cmake, meson, make, nil, etc.)
  - source: Where to get the source code (usually "." for the current directory)
  - build-packages: Packages needed at build time (from debian/control
    Build-Depends: field in the source stanza)
  - stage-packages: Packages installed into the build environment for staging
  - organize: Maps build output paths to package partition paths for
    multi-package sources
  - Other plugin-specific parameters

  The Build-Depends: field from debian/control maps to `build-packages` in
  parts. debcraft automatically generates the Build-Depends: field in the
  source control file from the union of all build-packages across all parts.

  The `organize` key is used to route files into specific binary packages via
  partition syntax: "(package/<pkgname>)/path".

  Example debian/control:
    Build-Depends: debhelper-compat (= 13), zlib1g-dev, build-essential

  Example debian/rules (simplified):
    %:
        dh $@
    override_dh_auto_configure:
        dh_auto_configure -- --prefix=/usr

  Example debcraft.yaml parts equivalent:
    parts:
      libpng1.6:
        plugin: autotools
        source: .
        autotools-configure-parameters:
          - --prefix=/usr
        build-packages:
          - build-essential
          - zlib1g-dev


package-repositories
--------------------
  Type: list[dict] | null (optional)

  Origin: No debian/control equivalent. Specifies additional APT package
  repositories to enable during the build. This is a craft-application feature.


PACKAGES SECTION (packages.<name>)
====================================

Each key under `packages:` corresponds to a binary package stanza in
debian/control. The key itself is the binary package name.

packages.<name> (key)
---------------------
  Origin: debian/control, `Package:` field in a binary package stanza.

  The binary package name. Must follow Debian package naming rules (same
  regex as the source name: /^[a-z0-9][a-z0-9.+-]+$/).

  Example debian/control:
    Package: libpng16-16t64

  Example debcraft.yaml:
    packages:
      libpng16-16t64:


packages.<name>.architectures
------------------------------
  Type: "any" | "all" | list[DebianArchitecture] (default: "any")

  Origin: debian/control, `Architecture:` field in the binary package stanza.

  "any" means the package is architecture-specific and will be built for each
  target architecture. "all" means it is architecture-independent. A list of
  specific architectures restricts building to those architectures only.

  Supported architectures: amd64, arm64, armhf, i386, ppc64el, riscv64, s390x.

  Example debian/control:
    Architecture: any      -> architectures: any
    Architecture: all      -> architectures: all
    Architecture: amd64 arm64  -> architectures: [amd64, arm64]

  At build time, the architecture value is determined by the build plan:
  - "any" resolves to the current build_for architecture
  - "all" resolves to "all"
  - A list is filtered to include only the current build_for architecture


packages.<name>.summary
------------------------
  Type: string | null (optional)

  Origin: debian/control, the short description (first line of `Description:`)
  in the binary package stanza.

  If null, falls back to the top-level project `summary`. At build time
  (gencontrol.py), if neither the package summary nor the project summary is
  set, an error is raised.

  Example debian/control:
    Description: PNG library - development (version 1.6)
     libpng is ...

  "PNG library - development (version 1.6)" is the package summary.

  Example debcraft.yaml:
    packages:
      libpng-dev:
        summary: PNG library - development (version 1.6)


packages.<name>.description
-----------------------------
  Type: string | null (optional)

  Origin: debian/control, the extended description (all lines after the first
  line of `Description:`) in the binary package stanza.

  In Debian control files, extended description lines are prefixed with a
  space, and blank paragraphs are represented as " .". In debcraft.yaml, this
  is a plain multi-line YAML string (using `|`); blank lines are empty lines.

  If null, falls back to the top-level project `description`. At build time,
  if neither the package nor project description is set, an error is raised.

  At build time the full Description field written to the control file is:
    Description: <summary>
     <description lines, each indented by one space>

  Example debcraft.yaml:
    packages:
      libpng-dev:
        description: |
          libpng is a library implementing an interface for reading and writing
          PNG (Portable Network Graphics) format files.

          This package contains the header and development files needed to
          build programs and packages using libpng.


packages.<name>.version
------------------------
  Type: string | null (optional)

  Origin: debian/control, `Version:` field in the binary package stanza (rare).
  More commonly from debian/changelog.

  If null, falls back to the top-level project `version`. At build time, if
  neither the package nor project version is set, an error is raised.

  In standard Debian packages the binary package version matches the source
  version. A separate per-package version is unusual and only needed when a
  binary package has a different version from the source.


packages.<name>.depends
------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Depends:` field in the binary package stanza.

  Each string is a Debian dependency specification, e.g.:
    "libc6 (>= 2.17)"
    "libpng16-16 (<< 1.6.43-5build1)"
    "${shlibs:Depends}"
    "${misc:Depends}"

  In traditional Debian packaging, `${shlibs:Depends}` and `${misc:Depends}`
  are substitution variables populated by dh_shlibdeps and dh_gencontrol. In
  debcraft, shared library dependencies are detected automatically by the
  shlibdeps helper (debcraft/helpers/shlibdeps.py) and merged with any
  user-specified depends. User-specified depends override auto-detected depends
  for the same package name (see gencontrol.py:_filter_dependencies()).

  The substitution variables ${shlibs:Depends} and ${misc:Depends} from
  debian/control are NOT used in debcraft.yaml; their functionality is
  provided automatically by debcraft's shlibdeps helper.

  Example debian/control:
    Depends: ${shlibs:Depends}, ${misc:Depends}, libpng16-16

  Example debcraft.yaml:
    packages:
      libpng-dev:
        depends:
          - libpng16-16t64 (= ${binary:Version})


packages.<name>.recommends
---------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Recommends:` field in the binary package stanza.

  Example debian/control:
    Recommends: some-package

  Example debcraft.yaml:
    packages:
      mypkg:
        recommends:
          - some-package


packages.<name>.suggests
-------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Suggests:` field in the binary package stanza.

  Example debian/control:
    Suggests: optional-package

  Example debcraft.yaml:
    packages:
      mypkg:
        suggests:
          - optional-package


packages.<name>.provides
-------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Provides:` field in the binary package stanza.

  Lists virtual packages that this package provides, allowing other packages
  to depend on the virtual name instead of a specific binary.

  Example debian/control:
    Provides: libpng16-16

  Example debcraft.yaml:
    packages:
      libpng16-16t64:
        provides:
          - libpng16-16


packages.<name>.breaks
-----------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Breaks:` field in the binary package stanza.

  Lists packages that this package breaks (makes non-functional). Usually
  paired with Replaces:. Used for smooth upgrades when a package is renamed
  or split.

  Example debian/control:
    Breaks: libpng16-16 (<< 1.6.43-5build1)

  Example debcraft.yaml:
    packages:
      libpng16-16t64:
        breaks:
          - libpng16-16 (<< 1.6.43-5build1)


packages.<name>.replaces
-------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Replaces:` field in the binary package stanza.

  Lists packages whose files this package replaces. Used when files have moved
  between packages. Usually paired with Breaks:.

  Example debian/control:
    Replaces: libpng16-16

  Example debcraft.yaml:
    packages:
      libpng16-16t64:
        replaces:
          - libpng16-16


packages.<name>.conflicts
--------------------------
  Type: list[string] | null (optional)

  Origin: debian/control, `Conflicts:` field in the binary package stanza.

  Lists packages that cannot be installed at the same time as this package.
  Stronger than Breaks: because it prevents co-installation entirely.

  Example debian/control:
    Conflicts: old-package-name

  Example debcraft.yaml:
    packages:
      mypkg:
        conflicts:
          - old-package-name


packages.<name>.section
------------------------
  Type: string | null (optional)

  Origin: debian/control, `Section:` field in the binary package stanza.

  Overrides the top-level project `section` for this specific package. At
  build time, if neither the package nor the project section is set, an error
  is raised.

  Example debian/control:
    Section: libdevel

  Example debcraft.yaml:
    packages:
      libpng-dev:
        section: libdevel


packages.<name>.multi-arch
---------------------------
  Type: "no" | "same" | "foreign" | "allowed" (default: "no")

  Origin: debian/control, `Multi-Arch:` field in the binary package stanza.

  Controls multi-architecture co-installation support:
  - "no": not multi-arch aware (default; Multi-Arch field is omitted entirely)
  - "same": multiple architecture variants can be co-installed (e.g., :amd64
    and :i386)
  - "foreign": can satisfy dependencies from any architecture (e.g., for
    architecture-independent data packages)
  - "allowed": package can be marked as Multi-Arch: same by the user

  When multi-arch is "no", the Multi-Arch field is omitted from the generated
  control file. For all other values, it is written as-is.

  Example debian/control:
    Multi-Arch: same

  Example debcraft.yaml:
    packages:
      libpng16-16t64:
        multi-arch: same


AUTOMATICALLY GENERATED FIELDS (not in debcraft.yaml)
======================================================

The following fields appear in the generated control files but are NOT set
in debcraft.yaml; they are computed automatically by debcraft at build time:

Standards-Version
  Generated by debcraft based on the version of debcraft being used.

Build-Depends
  Generated from the union of `build-packages` across all parts in all
  active partitions. (Source control file only.)

Build-Depends-Arch
  Generated from `build-packages` entries that use craft-grammar architecture
  conditionals. (Source control file only.)

Installed-Size
  Computed by the gencontrol helper as the sum of all file sizes in the
  package's prime directory, divided by 1024 (in kilobytes).

Package (in binary control)
  Taken from the key name under `packages:` in debcraft.yaml.

Source (in binary control)
  Taken from the top-level `name:` field in debcraft.yaml.

Architecture (in binary control)
  Resolved at build time from the package's `architectures` field and the
  current build plan (build_for value).


DEBIAN/ DIRECTORY FILES HANDLED BY HELPERS (not debcraft.yaml fields)
=======================================================================

The following debian/ files are read directly by debcraft helpers during the
build process. They do not correspond to debcraft.yaml fields. They can exist
in either debian/ or debcraft/ (debcraft/ takes precedence). Package-specific
versions are named <package-name>.<suffix>; a plain <suffix> file applies to
the main source package (project.name).

Data files (installed into binary package payload, handled by install helpers):
  debian/copyright                        -> Installed as usr/share/doc/<pkg>/copyright
                                             in all packages (installdocs helper). The
                                             debcraft/ version takes precedence over debian/.
  debian/changelog                        -> Installed as usr/share/doc/<pkg>/changelog.gz
                                             (native) or changelog.Debian.gz (non-native)
                                             in all packages (installchangelogs helper).
                                             The debcraft/ version takes precedence.
  debian/NEWS                             -> Installed as usr/share/doc/<pkg>/NEWS.Debian.gz
                                             in all packages (installchangelogs helper).
                                             The debcraft/ version takes precedence.
                                             Per-package NEWS files are not supported;
                                             use parts organize to route them manually.

Lintian overrides (installed into binary package payload):
  debian/lintian-overrides                -> usr/share/lintian/overrides/<project-name>
  debian/<pkg>.lintian-overrides          -> usr/share/lintian/overrides/<pkg>
                                             (lintian helper via install_package_data).
                                             The debcraft/ version takes precedence.

Auto-generated control files (not read from debian/):
  shlibs     -> Generated by makeshlibs helper by scanning ELF shared libraries in the
                prime dir. Not read from debian/.
  triggers   -> Generated by makeshlibs helper (activate-noawait ldconfig) when shared
                libraries are found. Not read from debian/.

Source package format:
  debian/source/format                    -> "3.0 (quilt)" or "3.0 (native)"
    Determines native vs. non-native packaging. debcraft reads this at build
    time to validate consistency with the version number.

Patches (for non-native packages with 3.0 (quilt) format):
  debian/patches/series                   -> Ordered list of patch files
  debian/patches/*.patch                  -> Individual patch files
    Patches are applied by craft-parts source handling, not debcraft directly.


MISSING FEATURES
================

This section lists debian/ files and debian/control fields that debcraft does
not yet support. This is intended to guide future implementation work.

Maintainer scripts (install_package_control not yet called)
-----------------------------------------------------------
The function install_package_control() exists in debcraft/helpers/helpers.py
and is capable of copying per-package control files from debian/ or debcraft/
into the binary package control tarball, but it is never called by any helper.
As a result, the following debian/ files are silently ignored:

  debian/postinst, debian/<pkg>.postinst  Post-installation maintainer script
  debian/preinst, debian/<pkg>.preinst    Pre-installation maintainer script
  debian/postrm, debian/<pkg>.postrm      Post-removal maintainer script
  debian/prerm, debian/<pkg>.prerm        Pre-removal maintainer script
  debian/config, debian/<pkg>.config      Debconf configuration script
  debian/templates, debian/<pkg>.templates Debconf templates
  debian/conffiles, debian/<pkg>.conffiles Explicit conffiles list

  To implement: wire up install_package_control() calls (analogous to how
  install_package_data() is called from the lintian helper) from a new
  installscripts helper or from the gencontrol helper.

File installation lists (no debcraft equivalent)
-------------------------------------------------
Traditional debhelper-based packaging uses debian/install files to list which
files from the build tree should be installed into each binary package and
where. debcraft has no equivalent mechanism; file routing must be done via
the `organize` key in parts, using partition syntax.

  debian/install, debian/<pkg>.install    File installation lists
  debian/docs, debian/<pkg>.docs          Documentation file lists (installdocs
                                          currently only handles copyright)
  debian/manpages, debian/<pkg>.manpages  Man page installation lists
  debian/examples, debian/<pkg>.examples  Example file installation lists
  debian/links, debian/<pkg>.links        Symlink creation lists
  debian/dirs, debian/<pkg>.dirs          Directory creation lists
  debian/info, debian/<pkg>.info          GNU info file installation lists

Per-package NEWS files (not implemented)
-----------------------------------------
The installchangelogs helper installs a single debian/NEWS to all packages.
Per-package debian/<pkg>.NEWS files are explicitly not implemented (see
comment in debcraft/helpers/installchangelogs.py). The workaround is to use
`organize` in parts to route NEWS files to the appropriate packages manually.

symbols files for packages being built (not read from debian/)
--------------------------------------------------------------
The shlibdeps helper reads symbols files from /var/lib/dpkg/info/ (for
installed system packages) and from the state dir (for packages being built
in the same source). It does NOT read debian/symbols or debian/<pkg>.symbols
from the source tree. In traditional packaging, debian/symbols is maintained
by the packager to declare exported symbols and establish tighter minimum
version constraints. Without this, debcraft's shlibdeps falls back to
shlibs-based dependency resolution, which gives coarser version bounds.

  debian/symbols, debian/<pkg>.symbols    Package symbols file

Additionally, the _SymbolMap parser in shlibdeps.py does not yet support
alternative library names (lines starting with `|` in symbols files).

Debug package splitting (strip helper)
--------------------------------------
The strip helper calls `strip --strip-unneeded` on ELF files but does not
split debug symbols into separate -dbgsym packages. In traditional packaging,
dh_strip creates a package-dbgsym binary package containing the unstripped
debug information. This means debcraft currently discards debug symbols
instead of preserving them in a separate package.

Source control fields with no debcraft.yaml equivalent
------------------------------------------------------
The following debian/control source stanza fields have no named debcraft.yaml
field and cannot be set without using the (currently removed) passthrough
mechanism:

  Standards-Version         Debian policy version the package conforms to
                            (auto-generated by debcraft)
  Build-Depends-Indep       Build dependencies only needed for arch-independent targets
  Build-Conflicts           Packages that conflict with the build environment
  Build-Conflicts-Arch      Architecture-specific build conflicts
  Build-Conflicts-Indep     Arch-independent build conflicts
  Rules-Requires-Root       Whether debian/rules needs root privileges
  Vcs-Git                   VCS repository URL (only Vcs-Browser maps to source-code)
  Testsuite                 Autopkgtest test suite declaration
  XS-*, XB-* fields         Custom source/binary extension fields

Binary package control fields with no debcraft.yaml equivalent
-------------------------------------------------------------
  Pre-Depends               Strict pre-installation dependencies
  Enhances                  Packages enhanced by installing this package
  Built-Using               Source packages included in this binary
  X-Cargo-Built-Using       Rust crate sources (Ubuntu-specific)
  Protected                 Prevents package removal (like Essential but weaker)
  Essential                 Essential system packages

Field validation gaps
---------------------
  maintainer    The Maintainer: field format (Name <email>) is not validated.
                See: https://github.com/canonical/debcraft/issues/39
  depends et al Dependency relationship strings (Depends:, Recommends:, etc.)
                are stored as plain strings without syntax validation.
                See: https://github.com/canonical/debcraft/issues/42


COMPLETE MAPPING SUMMARY TABLE
================================

debcraft.yaml field         | debian/ origin
----------------------------+-------------------------------------------------------
name                        | debian/control: Source:
version                     | debian/changelog: first entry version (between parens)
summary                     | debian/control: Description: short (first line)
description                 | debian/control: Description: extended (remaining lines)
maintainer                  | debian/control: Maintainer:
original-maintainer         | debian/control: XSBC-Original-Maintainer: / XS-Original-Maintainer:
uploaders                   | debian/control: Uploaders: (comma-separated -> list)
section                     | debian/control: Section: (source stanza)
priority                    | debian/control: Priority: (source stanza)
contact                     | debian/control: Maintainer: (approximate)
source-code                 | debian/control: Vcs-Browser:
license                     | debian/copyright: License: (primary SPDX identifier)
issues                      | (no debian/ equivalent)
adopt-info                  | (no debian/ equivalent)
base                        | debian/changelog: distribution codename (e.g. noble -> ubuntu@24.04)
build-base                  | (no debian/ equivalent)
platforms                   | debian/control: Architecture: (across all binary stanzas)
parts.<name>.build-packages | debian/control: Build-Depends: (source stanza)
parts                       | debian/rules + debian/patches/
package-repositories        | (no debian/ equivalent)
packages.<name> (key)       | debian/control: Package: (binary stanza)
packages.<name>.architectures | debian/control: Architecture: (binary stanza)
packages.<name>.summary     | debian/control: Description: short (binary stanza)
packages.<name>.description | debian/control: Description: extended (binary stanza)
packages.<name>.version     | debian/control: Version: (binary stanza, rare)
packages.<name>.depends     | debian/control: Depends: (binary stanza, without substitution vars)
packages.<name>.recommends  | debian/control: Recommends: (binary stanza)
packages.<name>.suggests    | debian/control: Suggests: (binary stanza)
packages.<name>.provides    | debian/control: Provides: (binary stanza)
packages.<name>.breaks      | debian/control: Breaks: (binary stanza)
packages.<name>.replaces    | debian/control: Replaces: (binary stanza)
packages.<name>.conflicts   | debian/control: Conflicts: (binary stanza)
packages.<name>.section     | debian/control: Section: (binary stanza)
packages.<name>.multi-arch  | debian/control: Multi-Arch: (binary stanza)
