//!Simple linux framebuffer abstraction.
//!Examples can be found [here](https://github.com/Roysten/rust-framebuffer/tree/master/examples).
use libc::ioctl;

use std::fmt;
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use errno::errno;
use memmap2::{MmapMut, MmapOptions};

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;
const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;

const KDSETMODE: libc::c_ulong = 0x4B3A;
const KD_TEXT: libc::c_ulong = 0x00;
const KD_GRAPHICS: libc::c_ulong = 0x01;

///Bitfield which is a part of VarScreeninfo.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Bitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

///Struct as defined in /usr/include/linux/fb.h
#[repr(C)]
#[derive(Clone, Debug)]
pub struct VarScreeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: Bitfield,
    pub green: Bitfield,
    pub blue: Bitfield,
    pub transp: Bitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

///Struct as defined in /usr/include/linux/fb.h Note: type is a keyword in Rust and therefore has been
///changed to fb_type.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct FixScreeninfo {
    pub id: [u8; 16],
    pub smem_start: usize,
    pub smem_len: u32,
    pub fb_type: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: usize,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

impl ::std::default::Default for Bitfield {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl ::std::default::Default for VarScreeninfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

impl ::std::default::Default for FixScreeninfo {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}

///Enum that can be used to set the current KdMode.
pub enum KdMode {
    Graphics = KD_GRAPHICS as isize,
    Text = KD_TEXT as isize,
}

///Kind of errors that can occur when dealing with the Framebuffer.
#[derive(Debug)]
pub enum FramebufferErrorKind {
    IoctlFailed,
    IoError,
}

#[derive(Debug)]
pub struct FramebufferError {
    pub kind: FramebufferErrorKind,
    pub details: String,
}

impl FramebufferError {
    fn new(kind: FramebufferErrorKind, details: &str) -> FramebufferError {
        FramebufferError {
            kind,
            details: String::from(details),
        }
    }
}

impl std::error::Error for FramebufferError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for FramebufferError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.details)
    }
}

impl std::convert::From<std::io::Error> for FramebufferError {
    fn from(err: std::io::Error) -> FramebufferError {
        FramebufferError::new(FramebufferErrorKind::IoError, &err.to_string())
    }
}

///Struct that should be used to work with the framebuffer. Direct usage of `frame` should not be
///necessary.
#[derive(Debug)]
pub struct Framebuffer {
    pub device: File,
    pub frame: MmapMut,
    pub var_screen_info: VarScreeninfo,
    pub fix_screen_info: FixScreeninfo,
}

impl Framebuffer {
    pub fn new<P: AsRef<Path>>(path_to_device: P) -> Result<Framebuffer, FramebufferError> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_to_device)?;

        let var_screen_info = Framebuffer::get_var_screeninfo(&device)?;
        let fix_screen_info = Framebuffer::get_fix_screeninfo(&device)?;

        let frame_length = (fix_screen_info.line_length * var_screen_info.yres_virtual) as usize;
        let frame = unsafe { MmapOptions::new().len(frame_length).map_mut(&device) };
        match frame {
            Ok(frame_result) => Ok(Framebuffer {
                device,
                frame: frame_result,
                var_screen_info,
                fix_screen_info,
            }),
            Err(_) => Err(FramebufferError::new(
                FramebufferErrorKind::IoError,
                &format!("Failed to map memory (offset: {} len: {})", 0, frame_length),
            )),
        }
    }

    ///Writes a frame to the Framebuffer.
    pub fn write_frame(&mut self, frame: &[u8]) {
        self.frame[..].copy_from_slice(frame);
    }

    ///Reads a frame from the framebuffer.
    pub fn read_frame(&self) -> &[u8] {
        &self.frame[..]
    }

    ///Creates a FixScreeninfo struct and fills it using ioctl.
    pub fn get_fix_screeninfo(device: &File) -> Result<FixScreeninfo, FramebufferError> {
        let mut info: FixScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_FSCREENINFO as _, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            _ => Ok(info),
        }
    }

    ///Creates a VarScreeninfo struct and fills it using ioctl.
    pub fn get_var_screeninfo(device: &File) -> Result<VarScreeninfo, FramebufferError> {
        let mut info: VarScreeninfo = Default::default();
        let result = unsafe { ioctl(device.as_raw_fd(), FBIOGET_VSCREENINFO as _, &mut info) };
        match result {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            _ => Ok(info),
        }
    }

    pub fn put_var_screeninfo(
        device: &File,
        screeninfo: &VarScreeninfo,
    ) -> Result<i32, FramebufferError> {
        match unsafe { ioctl(device.as_raw_fd(), FBIOPUT_VSCREENINFO as _, screeninfo) } {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            ret => Ok(ret),
        }
    }

    pub fn pan_display(device: &File, screeninfo: &VarScreeninfo) -> Result<i32, FramebufferError> {
        match unsafe { ioctl(device.as_raw_fd(), FBIOPAN_DISPLAY as _, screeninfo) } {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            ret => Ok(ret),
        }
    }

    ///Sets the tty graphics mode. Make sure to change it back to KdMode::Text after the program is
    ///done!
    pub fn set_kd_mode(kd_mode: KdMode) -> Result<i32, FramebufferError> {
        match unsafe { ioctl(0, KDSETMODE as _, kd_mode) } {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            ret => Ok(ret),
        }
    }

    /// Allows setting tty mode from non-terminal session by explicitly specifying device name
    pub fn set_kd_mode_ex<P: AsRef<Path>>(
        path_to_device: P,
        kd_mode: KdMode,
    ) -> Result<i32, FramebufferError> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path_to_device)?;

        match unsafe { ioctl(device.as_raw_fd(), KDSETMODE as _, kd_mode) } {
            -1 => Err(FramebufferError::new(
                FramebufferErrorKind::IoctlFailed,
                &format!("Ioctl returned -1: {}", errno()),
            )),
            ret => Ok(ret),
        }
    }
}
