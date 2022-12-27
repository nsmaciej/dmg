//! Attaching to existing disk images.

use std::env;
use std::io::{self, Cursor, ErrorKind};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use log::info;
use plist::Value;

static DISK_COMMAND: &str = "hdiutil";

#[derive(Debug, Clone)]
enum Mount {
    Default,
    Random(PathBuf),
    Root(PathBuf),
    Point(PathBuf),
}

/// Builder to attach a disk image.
#[derive(Debug, Clone)]
pub struct Attach {
    image: PathBuf,
    mount: Mount,
    hidden: bool,
    force_readonly: bool,
}

/// Data associated with an attached disk image.
#[derive(Debug)]
pub struct Info {
    /// Path at which the disk image is mounted.
    pub mount_point: PathBuf,

    /// Device node path for this disk image.
    pub device: PathBuf,
}

/// Convinience handle for detaching an attached disk image.
///
/// Created with [`Attach::attach()`]
#[derive(Debug)]
pub struct Handle(Info);

/// An attached disk image handle that detaches it when dropped.
///
/// Created with [`Attach::with()`]
#[derive(Debug)]
pub struct With(Info);

macro_rules! check {
    ($opt:expr) => {
        match $opt {
            Some(res) => res,
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "could not find property",
                ))
            }
        }
    };
}

/// Access the [`Info`] struct associated with this handle.
impl Deref for With {
    type Target = Info;
    fn deref(&self) -> &Info {
        &self.0
    }
}

/// Access the [`Info`] struct associated with this handle.
impl Deref for Handle {
    type Target = Info;
    fn deref(&self) -> &Info {
        &self.0
    }
}

impl Handle {
    /// Detach the image, ignoring any open files.
    pub fn force_detach(self) -> io::Result<()> {
        detach(&self.device, true)
    }

    /// Detach the image.
    pub fn detach(self) -> io::Result<()> {
        detach(&self.device, false)
    }
}

/// Detach the disk image on drop
impl Drop for With {
    fn drop(&mut self) {
        detach(&self.device, false).expect("could not detach");
    }
}

macro_rules! mount_fn {
    ($doc:expr, $name:ident, $variant:ident) => {
        #[doc=$doc]
        pub fn $name<P: Into<PathBuf>>(mut self, path: P) -> Attach {
            self.mount = Mount::$variant(path.into());
            self
        }
    };
}

impl Attach {
    /// Creates a new attach builder for the given disk image.
    pub fn new<P: Into<PathBuf>>(path: P) -> Attach {
        Attach {
            image: path.into(),
            mount: Mount::Default,
            hidden: false,
            force_readonly: false,
        }
    }

    mount_fn!(
        "Mount volumes on subdirectories of path instead of under `/Volumes`.",
        mount_root,
        Root
    );
    mount_fn!(
        "Asuming only one volume, mount it at path instead of in `/Volumes`.",
        mount_point,
        Point
    );
    mount_fn!(
        "Mount under `path` with a random unique mount point directory name.",
        mount_random,
        Random
    );

    /// Render the volume invisible in applications like Finder.
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Force the device to be read-only.
    pub fn force_readonly(mut self) -> Self {
        self.force_readonly = true;
        self
    }

    /// Mount in a random folder inside the temporary directory.
    ///
    /// Equivalent to `mount_random(std::env::temp_dir())`
    pub fn mount_temp(self) -> Attach {
        self.mount_random(env::temp_dir())
    }

    fn attach_info(self) -> io::Result<Info> {
        let mut cmd = Command::new(DISK_COMMAND);
        cmd.arg("attach");

        match self.mount {
            Mount::Default => {}
            Mount::Random(ref path) => {
                cmd.arg("-mountrandom");
                cmd.arg(path);
            }
            Mount::Root(ref path) => {
                cmd.arg("-mountroot");
                cmd.arg(path);
            }
            Mount::Point(ref path) => {
                cmd.arg("-mountpoint");
                cmd.arg(path);
            }
        }

        if self.force_readonly {
            cmd.arg("-readonly");
        }

        if self.hidden {
            cmd.arg("-nobrowse");
        }

        cmd.arg("-plist");
        cmd.arg(&self.image);

        info!("Attaching {cmd:?}");
        let output = cmd.output()?;
        info!("Status {:?}", output.status);

        if !output.status.success() {
            // This is not as informative as I wish it would be
            // .. but neither is hdiutil
            let stderr = String::from_utf8(output.stderr)
                .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
            return Err(io::Error::new(
                ErrorKind::Other,
                format!("hdiutil attach failed: {stderr}"),
            ));
        }

        if let Ok(plist) = Value::from_reader(Cursor::new(output.stdout)) {
            let entities =
                check!(check!(check!(plist.as_dictionary()).get("system-entities")).as_array());
            for entity in entities {
                let properties = check!(entity.as_dictionary());
                if let Some(mount_point) = properties.get("mount-point") {
                    return Ok(Info {
                        mount_point: PathBuf::from(check!(mount_point.as_string())),
                        // If we don't have this something has gonne _really_ wrong
                        device: PathBuf::from(check!(properties["dev-entry"].as_string())),
                    });
                }
            }
            return Err(io::Error::new(ErrorKind::Other, "could not extract data"));
        }
        Err(io::Error::new(
            ErrorKind::InvalidData,
            "could not parse plist",
        ))
    }

    /// Attach the disk image
    pub fn attach(self) -> io::Result<Handle> {
        self.attach_info().map(Handle)
    }

    /// Attach the disk image, detaching when dropped
    pub fn with(self) -> io::Result<With> {
        self.attach_info().map(With)
    }
}

/// Detach an image using a path.
///
/// The path can be either a device node path or a mount point.
pub fn detach<P: AsRef<Path>>(path: P, force: bool) -> io::Result<()> {
    let mut cmd = Command::new(DISK_COMMAND);
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    cmd.arg("detach");
    if force {
        cmd.arg("-force");
    }
    cmd.arg(path.as_ref());

    info!("Detaching (force: {:?}): {:?}", force, cmd);
    let status = cmd.status()?;
    info!("Status {:?}", status);

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            ErrorKind::Other,
            "non-zero exit status for detach",
        ))
    }
}
