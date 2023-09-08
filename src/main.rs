use chrono::prelude::*;
use chrono::Duration;
use clap::Parser;
use gpsd_proto::{get_data, handshake, ResponseData};
// use log::{debug, info, warn};
use std::fs::File;
use std::io::{Write, BufReader, BufWriter};
use std::net::TcpStream;
use std::thread::sleep;
// use std::time::Duration;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[clap(version = "1.0", author = "Julius Koskela")]
struct Cli {
    /// Output path for the GPS data
    #[clap(required = true, short, long)]
    output_path: PathBuf,

    /// Interval between GPS data points in seconds
    #[clap(short, long)]
    interval: Option<u64>,

    /// Time in milliseconds that is spent listening for GPS data for one observation.
    #[clap(short, long)]
    gps_search_duration: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct GpsData {
    device: Option<ResponseData>,
    tpv: Option<ResponseData>,
    sky: Option<ResponseData>,
    pps: Option<ResponseData>,
    gst: Option<ResponseData>,
}

impl GpsData {
    fn new() -> Self {
        Self {
            device: None,
            tpv: None,
            sky: None,
            pps: None,
            gst: None,
        }
    }

    fn receive(&mut self, msg: ResponseData) {
        match msg {
            ResponseData::Device(ref _d) => {
                self.device = Some(msg);
            }
            ResponseData::Tpv(ref _t) => {
                self.tpv = Some(msg);
            }
            ResponseData::Sky(ref _sky) => {
                self.sky = Some(msg);
            }
            ResponseData::Pps(ref _p) => {
                self.pps = Some(msg);
            }
            ResponseData::Gst(ref _g) => {
                self.gst = Some(msg);
            }
        }
    }

    fn observe(&mut self, reader: &mut BufReader<&TcpStream>, duration: &chrono::Duration) {
        let now = Utc::now();
        while Utc::now() - now < *duration {
            let msg = get_data(reader).unwrap();
            self.receive(msg);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GpsDataVec(Vec<GpsData>);

impl GpsDataVec {
    fn append(&mut self, gps_data: GpsData) {
        self.0.push(gps_data);
    }

    fn to_json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }

    fn read_file(file: &mut File) -> Self {
        let mut reader = std::io::BufReader::new(file);
        let gps_data_vec: GpsDataVec = serde_json::from_reader(&mut reader).unwrap();
        gps_data_vec
    }

    fn write_file(&self, file: &mut File) {
        write!(file, "{}", self.to_json()).unwrap();
    }
}

struct GpsRecorder {
    stream: TcpStream,
    file: File,
    interval: Duration,
    gps_search_duration: Duration,
}

impl GpsRecorder {
    fn new(
        hostname: &str,
        file: &PathBuf,
        interval: u64,
        gps_search_duration: u64,
    ) -> Self {
        let stream = if let Ok(stream) = TcpStream::connect(hostname) {
            stream
        } else {
            eprintln!("Error connecting to GPSD");
            std::process::exit(1);
        };
        let file = File::create(file).unwrap();
        Self {
            stream,
            file,
            interval: Duration::seconds(interval as i64),
            gps_search_duration: Duration::milliseconds(gps_search_duration as i64),
        }
    }

    fn record(&mut self) {
        let mut reader = BufReader::new(&self.stream);
        let mut writer = BufWriter::new(&self.stream);
        handshake(&mut reader, &mut writer).unwrap();
        loop {
            let now = Utc::now();
            let mut gps_data = GpsData::new();
            let mut gps_data_vec = GpsDataVec::read_file(&mut self.file);
            gps_data.observe(&mut reader, &self.gps_search_duration);
            gps_data_vec.append(gps_data);
            gps_data_vec.write_file(&mut self.file);
            let remaining_time = self.interval - (Utc::now() - now);
            sleep(remaining_time.to_std().unwrap());
        }
    }
}

fn main() {
    let args = Cli::parse();

    let filename = format!("{}_GPS_data.csv", Utc::now().format("%Y-%m-%dT%H-%M-%S"));
    let file_path = args.output_path.join(filename.clone());

    let interval = args.interval.unwrap_or(10);
    let gps_search_duration = args.gps_search_duration.unwrap_or(1000);

    let mut recorder = GpsRecorder::new(
        "localhost:2947",
        &file_path,
        interval,
        gps_search_duration,
    );
    recorder.record();
}
