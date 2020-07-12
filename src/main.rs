use ifstructs;
use nix;
use nix::libc;
use std::os::unix::io::FromRawFd;

use tokio::prelude::*;
use tokio::io::AsyncWriteExt;

const TUN_IOC_MAGIC: u8 = b'T';
const TUN_SET_IFF: u8 = 202;

nix::ioctl_write_ptr_bad!(create_tuntap, nix::request_code_write!(TUN_IOC_MAGIC, TUN_SET_IFF, std::mem::size_of::<libc::c_int>()), ifstructs::ifreq);

struct TapDevice {
    fd: i32,
    name: String
}

impl TapDevice {
    pub fn new(name: &str) -> Result<TapDevice, nix::Error> {
        let mut ioctl_flags = ifstructs::ifreq::from_name(name).unwrap();
        ioctl_flags.set_flags((libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short);

        let fd = nix::fcntl::open("/dev/net/tun", nix::fcntl::OFlag::O_RDWR, nix::sys::stat::Mode::empty())?;
        unsafe { create_tuntap(fd, &mut ioctl_flags as *mut ifstructs::ifreq) }
            .and_then(|_| Ok(TapDevice {
                fd,
                name: ioctl_flags.get_name().unwrap()
            }))
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn split(self) -> (TapDeviceReader, TapDeviceWriter) {
        let reader_file = tokio::fs::File::from_std(unsafe { std::fs::File::from_raw_fd(self.fd) });
        let writer_file = tokio::fs::File::from_std(unsafe { std::fs::File::from_raw_fd(self.fd) });
        (
            TapDeviceReader{file: reader_file},
            TapDeviceWriter{file: writer_file}
        )
    }
}

struct TapDeviceReader {
    file: tokio::fs::File
}

impl TapDeviceReader {
    pub async fn read(&mut self, buf: &mut [u8]) -> tokio::io::Result<usize> {
        self.file.read(buf).await
    }
}

struct TapDeviceWriter {
    file: tokio::fs::File
}

impl TapDeviceWriter {
    pub async fn write(&mut self, buf: & [u8]) -> tokio::io::Result<()> {
        self.file.write_all(buf).await
    }
}

async fn tap_reader(mut reader: TapDeviceReader) {
    let mut buf = [0u8; 1600];
    'main_loop: loop {
        let amount_read = reader.read(&mut buf).await;
        let ether = etherparse::SlicedPacket::from_ethernet(&buf).unwrap();
        let link = ether.link.unwrap();
        match link {
            etherparse::LinkSlice::Ethernet2(eth) => {
                println!("Ethernet type = {:x}", eth.ether_type());
                if eth.ether_type() == 0x0806 {
                    println!("Destination = {:x?}", eth.destination());
                    println!("Source = {:x?}", eth.source());
                }
            }
        };
        match amount_read {
            Ok(sz) => println!("Read {} bytes", sz),
            Err(_) => break 'main_loop
        };
    }
}

async fn tap_writer(mut writer: TapDeviceWriter, mut data_channel: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>) {
    'main_loop: loop {
        let data = data_channel.recv().await;
        match data {
            None => break 'main_loop,
            Some(bytes) => writer.write(bytes.as_slice()).await.unwrap()
        };
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().map(|x| x.to_string()).collect();
    if args.len() > 1 {
        if args[1] == "ports" {
            tokio_serial
        }
    }
    let dev = TapDevice::new("").unwrap();
    println!("Created device {}", dev.get_name());
    let (reader, writer) = dev.split();

    let (_sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Vec::<u8>>();

    let tap_reader_handle = tokio::task::spawn(tap_reader(reader));
    let tap_writer_handle = tokio::task::spawn(tap_writer(writer, receiver));
    
    tap_reader_handle.await.unwrap();
    tap_writer_handle.await.unwrap();
}
