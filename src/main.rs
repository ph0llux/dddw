// - STD
use std::process::exit;
use std::path::PathBuf;
use std::error::Error;
use std::ptr;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::{cmp::min, fmt::Write as OtherWrite};

// - modules
#[cfg(target_family = "windows")]
mod listdevices;
#[cfg(target_family = "windows")]
mod dump;
mod constants;
mod traits;

// - internal
#[cfg(target_family = "windows")]
use listdevices::*;
#[cfg(target_family = "windows")]
use dump::*;
use constants::*;
use traits::*;

// - types
type Result<T> = std::result::Result<T, Box<dyn Error>>;

// - external
use clap::{
    Parser,
    Subcommand,
    ValueEnum,
};
use log::{LevelFilter, error};
use indicatif::{ProgressBar, ProgressState, ProgressStyle, HumanBytes};

#[cfg(target_family = "windows")]
use comfy_table::{
    Table, Attribute, Cell, ContentArrangement,
    modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL,
};

#[cfg(target_family = "windows")]
use windows_drives::drive::{BufferedPhysicalDrive, BufferedHarddiskVolume};
#[cfg(target_family = "windows")]
use winapi::{
    shared::{
        minwindef::{MAX_PATH, DWORD},
    },
    um::{
        fileapi::{
            GetVolumeInformationByHandleW, 
            FindFirstVolumeW,
            FindNextVolumeW,
            QueryDosDeviceW,
            GetVolumePathNamesForVolumeNameW,
            OPEN_EXISTING,
            CreateFileW,
        },
        handleapi::{INVALID_HANDLE_VALUE, CloseHandle},
        ioapiset::DeviceIoControl,
        winioctl::{IOCTL_STORAGE_GET_DEVICE_NUMBER, STORAGE_DEVICE_NUMBER, IOCTL_DISK_GET_DRIVE_GEOMETRY, DISK_GEOMETRY},
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, HANDLE},
    },
};

#[derive(Parser)]
#[clap(about, version, author, override_usage="dddw <SUBCOMMAND> [OPTIONS]")]
struct Cli {
    /// The Loglevel
    #[clap(short='L', long="log-level", value_enum, default_value="info", global=true, required=false)]
    log_level: LogLevel,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// acquire a physical image
    #[clap(arg_required_else_help=true)]
    Dump {
        /// The input file. This should be your device to dump. This field is REQUIRED.
        /// You can use all devices which are available by using the "dddw list-devices" command.
        /// On windows systems, you can either use the device names which are listed by the "dddw list-devices" command or the full device path,
        /// e.g. "disk0" for "\\.\PhysicalDrive0" or "volume1" for "\\.\HarddiskVolume1".
        #[clap(short='i', long="inputfile", required=true)]
        inputfile: PathBuf,

        /// The the name/path of the output-file WITHOUT file extension. E.g. "/home/ph0llux/sda_dump". File extension will be added automatically. This field is REQUIRED.
        #[clap(short='o', long="outputfile", global=true, required=true)]
        outputfile: String,
    },

    #[cfg(target_family = "windows")]
    /// List all available physical devices,
    /// which can be used as input for the physical subcommand.
    #[clap()]
    ListDevices { },
}

#[derive(ValueEnum, Clone, PartialEq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    FullInfo,
    Debug,
    FullDebug,
    Trace
}



#[cfg(target_family = "windows")]
fn main() {
    let args = Cli::parse();

    let log_level = match args.log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::FullInfo => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::FullDebug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };
    if args.log_level == LogLevel::FullInfo || args.log_level == LogLevel::FullDebug {
        env_logger::builder()
        .format_timestamp_nanos()
        .filter_level(log_level)
        .init();
    } else {
        env_logger::builder()
        .format_timestamp_nanos()
        .filter_module(env!("CARGO_PKG_NAME"), log_level)
        .init();
    };

    match args.command {
        Commands::ListDevices {  } => {
            print_devices_table();
            exit(EXIT_STATUS_SUCCESS);
        },
        Commands::Dump { inputfile, outputfile } => {
            let (input, size) = open_physical_drive(inputfile).unwrap();
            match dump(input, outputfile, size) {
                Ok(_) => {
                    exit(EXIT_STATUS_SUCCESS);
                },
                Err(e) => {
                    error!("{}", e);
                    exit(EXIT_STATUS_ERROR);
                }
            }
        },
    }
}

#[cfg(target_family = "windows")]
fn dump<R: Read>(mut input: R, outputfile: String, size: u64) -> Result<()> {
    use std::io::BufRead;

    let mut output = File::create(outputfile.clone() + ".img")?;

    let pb = ProgressBar::new(size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}")
        .unwrap()
        .progress_chars("#>-"));

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_read = 0;
    loop {
        let r = match input.read(&mut buffer) {
            Ok(r) => r,
            Err(e) => match e.kind() {
                std::io::ErrorKind::Interrupted => continue,
                _ => return Err(Box::new(e)),
            },
        };
        if r == 0 {
            break;
        }
        bytes_read += r;
        pb.inc(r as u64);
        output.write_all(&buffer[..r])?;
    }

    pb.finish_with_message("done");

    Ok(())
}