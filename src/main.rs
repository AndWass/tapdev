use tokio::io::{ReadHalf, WriteHalf};
use tokio::prelude::*;

mod dataframer;
mod framed;
mod serialframe;
mod tapdevice;

async fn simplex_connect<
    Source: io::AsyncRead + Unpin,
    Dest: io::AsyncWrite + Unpin,
    InputFramer: dataframer::Framer,
    OutputFramer: dataframer::Framer,
>(
    mut source: framed::Source<Source, InputFramer>,
    mut destination: framed::Destination<Dest, OutputFramer>,
) {
    let mut buffer = [0u8; 1600];
    'main_loop: loop {
        let read_result = source.read(&mut buffer).await;
        match read_result {
            Ok(values) => {
                for value in &values {
                    destination.write(value.as_slice()).await.unwrap();
                }
            }
            Err(_) => break 'main_loop,
        }
    }
}

fn framed_split<T: AsyncRead + AsyncWrite, F: dataframer::Framer>(
    stream: T,
    framer: F,
) -> (
    framed::Source<ReadHalf<T>, F>,
    framed::Destination<WriteHalf<T>, F>,
) {
    use framed::{Destination, Source};
    let split = io::split(stream);
    (
        Source::new(split.0, framer.clone()),
        Destination::new(split.1, framer),
    )
}

fn connect_spawn<Source, Dest, SourceFramer, DestFramer>(
    source: Source,
    destination: Dest,
    source_framer: SourceFramer,
    dest_framer: DestFramer,
) -> (tokio::task::JoinHandle<()>, tokio::task::JoinHandle<()>)
where
    Source: io::AsyncRead + io::AsyncWrite + Unpin + Send + 'static,
    Dest: io::AsyncRead + io::AsyncWrite + Unpin + Send + 'static,
    SourceFramer: dataframer::Framer + Send + 'static,
    DestFramer: dataframer::Framer + Send + 'static,
{
    let split_source = framed_split(source, source_framer);
    let split_dest = framed_split(destination, dest_framer);

    use tokio::task;
    let source_to_dest = simplex_connect(split_source.0, split_dest.1);
    let first = task::spawn(source_to_dest);

    let dest_to_source = simplex_connect(split_dest.0, split_source.1);
    let second = task::spawn(dest_to_source);

    (first, second)
}

#[tokio::main]
async fn main() {
    use tapdevice::TapDevice;

    let args: Vec<String> = std::env::args().map(|x| x).collect();
    if args.len() != 2 {
        println!("Usage: tapdev <serialport>");
        return;
    }
    let tap = TapDevice::new("").unwrap();
    println!("Created device {}", tap.get_name());

    let tapdev = tokio::io::PollEvented::new(tap).unwrap();

    let mut serial_settings = tokio_serial::SerialPortSettings::default();
    serial_settings.baud_rate = 115200;
    let serial = tokio_serial::Serial::from_path(&args[1], &serial_settings).unwrap();

    let tasks = connect_spawn(
        tapdev,
        serial,
        dataframer::Identity {},
        serialframe::Framer::new(),
    );

    tasks.0.await.unwrap();
    tasks.1.await.unwrap();
}
