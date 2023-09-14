use chrono::prelude::*;
use chrono::Duration;
use clap::Parser;
use gpsd_proto::{get_data, handshake, ResponseData};
// use log::{debug, info, warn};
use std::fs::File;
use std::io::Read;
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version = "1.0", author = "Julius Koskela")]
struct Cli {
    /// Output path for the GPS data
    #[clap(required = true, short, long)]
    output_path: PathBuf,

    /// Hostname of the GPSD server
    #[clap(short, long)]
    hostname: Option<String>,

    /// Port of the GPSD server
    #[clap(short, long)]
    port: Option<u16>,

    /// Interval between GPS data points in seconds
    #[clap(short, long)]
    interval: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct GpsData {
    timestamp: DateTime<Utc>,
    device: Option<ResponseData>,
    tpv: Option<ResponseData>,
    sky: Option<ResponseData>,
    pps: Option<ResponseData>,
    gst: Option<ResponseData>,
}

impl GpsData {
    fn new() -> Self {
        Self {
            timestamp: Utc::now(),
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
            match get_data(reader) {
                Ok(msg) => self.receive(msg),
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    continue;
                }
            }
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

    fn read_file(file: &PathBuf) -> Self {
        let mut file = File::open(file).unwrap();
        let mut data_string = String::new();
        file.read_to_string(&mut data_string).unwrap();
        if data_string.is_empty() {
            return Self(Vec::new());
        }
        let gps_data_vec: GpsDataVec = serde_json::from_str(&mut data_string).unwrap();
        gps_data_vec
    }

    fn write_file(&self, file: &PathBuf) {
        let mut file = File::create(file).unwrap();
        write!(file, "{}", self.to_json()).unwrap();
    }
}

struct GpsRecorder {
    stream: TcpStream,
    file_path: PathBuf,
    interval: Duration,
}

impl GpsRecorder {
    fn new(hostname: &str, path: &PathBuf, interval: u64) -> Self {
        let stream = if let Ok(stream) = TcpStream::connect(hostname) {
            stream
        } else {
            eprintln!("Error connecting to GPSD");
            std::process::exit(1);
        };
        let filename = format!("{}_GPS_data.json", Utc::now().format("%Y-%m-%dT%H-%M-%S"));
        let file_path = path.join(filename);
        File::create(&file_path).unwrap();
        Self {
            stream,
            file_path,
            interval: Duration::seconds(interval as i64),
        }
    }

    fn record(&mut self) {
        let mut reader = BufReader::new(&self.stream);
        let mut writer = BufWriter::new(&self.stream);
        handshake(&mut reader, &mut writer).unwrap();
        loop {
            let mut gps_data = GpsData::new();
            let mut gps_data_vec = GpsDataVec::read_file(&self.file_path);
            gps_data.observe(&mut reader, &self.interval);
            gps_data_vec.append(gps_data);
            gps_data_vec.write_file(&self.file_path);
        }
    }
}

fn main() {
    let args = Cli::parse();

    let interval = args.interval.unwrap_or(10);

    let mut recorder = GpsRecorder::new(
        "localhost:2947",
        &args.output_path,
        interval,
    );
    recorder.record();
}
