use nix::libc;

use mio::unix::EventedFd;
use mio::Evented;
use mio::{Poll, PollOpt, Ready, Token};
use std::io;

const TUN_IOC_MAGIC: u8 = b'T';
const TUN_SET_IFF: u8 = 202;

nix::ioctl_write_ptr_bad!(
    create_tuntap,
    nix::request_code_write!(
        TUN_IOC_MAGIC,
        TUN_SET_IFF,
        std::mem::size_of::<libc::c_int>()
    ),
    ifstructs::ifreq
);

pub struct TapDevice {
    handle: i32,
    name: String,
}

impl TapDevice {
    pub fn new(name: &str) -> Result<TapDevice, nix::Error> {
        let mut ioctl_flags = ifstructs::ifreq::from_name(name).unwrap();
        ioctl_flags.set_flags((libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short);

        let fd = nix::fcntl::open(
            "/dev/net/tun",
            nix::fcntl::OFlag::O_RDWR | nix::fcntl::OFlag::O_NONBLOCK,
            nix::sys::stat::Mode::empty(),
        )?;
        unsafe { create_tuntap(fd, &mut ioctl_flags as *mut ifstructs::ifreq) }.and_then(|_| {
            Ok(TapDevice {
                handle: fd,
                name: ioctl_flags.get_name().unwrap(),
            })
        })
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
}

impl Evented for TapDevice {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.handle).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.handle).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.handle).deregister(poll)
    }
}

impl io::Read for TapDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amount_read = unsafe {
            nix::libc::read(
                self.handle,
                buf.as_mut_ptr() as *mut nix::libc::c_void,
                buf.len(),
            )
        };

        if amount_read >= 0 {
            return Ok(amount_read as usize);
        }
        Err(io::Error::from_raw_os_error(nix::errno::errno()))
    }
}

impl io::Write for TapDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let amount_written = unsafe {
            nix::libc::write(
                self.handle,
                buf.as_ptr() as *const nix::libc::c_void,
                buf.len(),
            )
        };
        if amount_written >= 0 {
            return Ok(amount_written as usize);
        }
        Err(io::Error::from_raw_os_error(nix::errno::errno()))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
