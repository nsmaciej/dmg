//! Creating new disk images.

use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::Command;

use log::info;
use tempfile;

static DISK_COMMAND: &str = "hdiutil";

macro_rules! format_enum {
    (
        $(#[$($enum_attrs:tt)*])*
        pub enum $name:ident {
            $( $(#[$($variant_attrs:tt)*])* $variant:ident, )*
        }
    ) => {
        $(#[$($enum_attrs)*])*
        pub enum $name {
            $( $(#[$($variant_attrs)*])* $variant, )*
        }
        impl $name {
            #[allow(deprecated)]
            fn format_name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

format_enum! {
    /// Format that can be passed to [`FromFolder`]
    #[non_exhaustive]
    #[derive(Debug, Clone)]
    pub enum FolderImageFormat {
        /// read-only
        UDRO,
        /// compressed (ADC)
        UDCO,
        /// compressed
        UDZO,
        /// compressed (bzip2)
        #[deprecated = "Marked as deprecated in `hdiutil create -help`"]
        UDBZ,
        /// compressed (lzfse)
        ULFO,
        /// compressed (lzma)
        ULMO,
        /// entire device
        UFBI,
        /// iPod image
        IPOD,
        /// sparsebundle
        UDSB,
        /// sparse
        UDSP,
        /// read/write
        UDRW,
        /// DVD/CD master
        UDTO,
        /// hybrid image (HFS+/ISO/UDF)
        UNIV,
        /// sparse bundle disk image
        SPARSEBUNDLE,
        /// sparse disk image
        SPARSE,
        /// read/write disk image
        UDIF,
    }
}

/// Options common between different `hdiutil create` modes.
#[derive(Debug, Clone)]
struct CommonOptions {
    overwrite: bool,
    volume_name: Option<OsString>,
    // Missing options, not sure if they are worth implementing:
    // -layout
    // -partitionType
    // -align
    // -fs
    // -stretch
}

/// Builder to create a disk image from a source folder.
#[derive(Debug, Clone)]
pub struct FromFolder {
    common_options: CommonOptions,
    source_folder: PathBuf,
    spotlight: bool,
    any_owners: bool,
    skip_unreadable: bool,
    atomic: bool,
    //TODO: Add srcowners.
    format: FolderImageFormat,
}

/// Opaque struct which deletes the disk image when dropped.
///
/// Created with [`FromFolder::create_temp`]
#[derive(Debug)]
pub struct TempImagePath(tempfile::TempPath);

impl Deref for TempImagePath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for TempImagePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<OsStr> for TempImagePath {
    fn as_ref(&self) -> &OsStr {
        self.0.as_ref()
    }
}

fn apply_common_options(cmd: &mut Command, options: &CommonOptions) {
    if options.overwrite {
        cmd.arg("-ov");
    }
    if let Some(name) = &options.volume_name {
        cmd.arg("-volname");
        cmd.arg(name);
    }
}

fn binary_option(cmd: &mut Command, option: &str, enabled: bool) {
    cmd.arg(if enabled {
        format!("-{option}")
    } else {
        format!("-no{option}")
    });
}

macro_rules! common_options_build {
    () => {
        /// Overwrite (clobber) an existing file.
        pub fn overwrite(mut self) -> Self {
            self.common_options.overwrite = true;
            self
        }

        /// Set the volume name of the disk image. The default depends the
        /// filesystem being used; the default volume name in both HFS+ and APFS
        /// is "untitled". When creating an image from a source folder, the source
        /// folder's name appears to be used.
        pub fn volume_name(mut self, name: impl Into<OsString>) -> Self {
            self.common_options.volume_name = Some(name.into());
            self
        }
    };
}

impl FromFolder {
    /// Create a new builder for creating a disk image from a source folder.
    pub fn new(source_folder: impl Into<PathBuf>) -> Self {
        Self {
            common_options: CommonOptions {
                overwrite: false,
                volume_name: None,
            },
            source_folder: source_folder.into(),
            // We are assuming these are false by default.
            // If necessary we can always make them Options.
            spotlight: false,
            any_owners: false,
            // These follow defaults from `hdiutil create -help`.
            skip_unreadable: false,
            atomic: true,
            format: FolderImageFormat::UDZO,
        }
    }

    common_options_build!();

    /// Skip files that can't be read by the copying user and don't authenticate.
    pub fn skip_unreadable(mut self) -> Self {
        self.skip_unreadable = true;
        self
    }

    /// Do not fail if the user invoking hdiutil can't ensure correct file
    /// ownership for the files in the image.
    pub fn any_owners(mut self) -> Self {
        self.any_owners = true;
        self
    }

    /// Create a Spotlight index.
    pub fn spotlight_index(mut self) -> Self {
        self.spotlight = true;
        self
    }

    /// Do not copy files to a temporary location and then rename them to their destination.
    /// May be slightly faster.
    pub fn non_atomic(mut self) -> Self {
        self.atomic = false;
        self
    }

    /// Create a disk image with a given path.
    pub fn create(self, image_path: impl Into<PathBuf>) -> io::Result<()> {
        let mut cmd = Command::new(DISK_COMMAND);
        cmd.arg("create");
        apply_common_options(&mut cmd, &self.common_options);

        binary_option(&mut cmd, "spotlight", self.spotlight);
        binary_option(&mut cmd, "anyowners", self.any_owners);
        binary_option(&mut cmd, "skipunreadable", self.skip_unreadable);
        binary_option(&mut cmd, "atomic", self.atomic);

        cmd.arg("-format");
        cmd.arg(self.format.format_name());

        cmd.arg("-srcfolder");
        cmd.arg(self.source_folder);

        cmd.arg(image_path.into());
        info!("Creating {cmd:?}");
        let output = cmd.output()?;
        info!("Status {:?}", output.status);

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            return Err(io::Error::new(
                ErrorKind::Other,
                format!("hdiutil create failed: {stderr}"),
            ));
        }

        Ok(())
    }

    /// Create a disk image in a temporary directory. The resulting disk image is
    /// deleted when [`TempImagePath`] is dropped. Useful for unit tests.
    pub fn create_temp(self) -> io::Result<TempImagePath> {
        let temp_path = tempfile::Builder::new()
            .suffix(".dmg")
            .tempfile()?
            .into_temp_path();
        self.overwrite() // Required since tempfile created the file.
            .create(temp_path.to_owned())?;
        Ok(TempImagePath(temp_path))
    }
}
