use std::time::Duration;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::time::Instant;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

mod compressor;
use crate::compressor::{Brotli, Compressor, Gzip, Lz4, Snappy, Zstd};

#[derive(clap::ArgEnum, Clone, Debug)]
enum Algorithm {
  All,
  Brotli,
  Gzip,
  Lz4,
  Snappy,
  Zstd,
}

#[derive(clap::ArgEnum, Clone, Debug)]
enum Operation {
  Benchmark,
  Compress,
  Decompress,
}

// Compress/Decompress with popular algorithms or compare performance
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  // Algorithm to use
  #[clap(arg_enum, short, long, default_value_t = Algorithm::All)]
  algorithm: Algorithm,
  // Path to file to operate on
  #[clap(short, long, parse(from_os_str))]
  file: std::path::PathBuf,
  // Operation to perform
  #[clap(arg_enum, short, long, default_value_t = Operation::Benchmark)]
  operation: Operation,
  #[clap(short, long, action)]
  base64: bool,
  // number of iterations to run for benchmarking
  #[clap(short, long, default_value_t = 25)]
  iterations: u32,
}

fn elapsed_secs(elapsed: Duration) -> f64 {
  elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
}

fn format_size(bytes: usize) -> String {
  if bytes < 1024 {
    format!("{} B", bytes)
  } else if bytes < 1024 * 1024 {
    format!("{:.1} KB", bytes as f64 / 1024_f64)
  } else if bytes < 1024 * 1024 * 1024 {
    format!("{:.2} MB", bytes as f64 / 1048576_f64)
  } else {
    format!("{:.2} GB", bytes as f64 / 1073741824_f64)
  }
}

fn main() {
  let args = Args::parse();

  let mut f = File::open(args.file.clone()).unwrap();
  let mut buffer = Vec::new();
  match args.base64 {
    true => {
      match args.operation {
        Operation::Decompress => {
          let mut b64_reader = base64::read::DecoderReader::new(&mut f, base64::STANDARD);
          b64_reader.read_to_end(&mut buffer).unwrap();
        },
        _ => {
          f.read_to_end(&mut buffer).unwrap();
        }
      }
    },
    false => {
      f.read_to_end(&mut buffer).unwrap();
    }
  }

  let mut b64_writer = base64::write::EncoderWriter::new(io::stdout(), base64::STANDARD);

  match args.operation {
    Operation::Benchmark => {
      let size_in_mb = buffer.len() as f64 / 1048576_f64;
      println!("Running {} iterations of all algorithms on {:?}", args.iterations, args.file);
      println!("Input size: {}", format_size(buffer.len()));

      let mut ratio = 100.0;

      let mut algs: Vec<Box<dyn Compressor>> = Vec::new();
      algs.push(Box::new(Brotli::new()));
      algs.push(Box::new(Gzip::new()));
      algs.push(Box::new(Lz4::new()));
      algs.push(Box::new(Snappy::new()));
      algs.push(Box::new(Zstd::new()));

      for alg in algs {
        println!();
        let progress = ProgressBar::new(args.iterations as u64);
        progress.set_style(
          ProgressStyle::default_bar()
            .template("{msg} {wide_bar:.cyan/blue} {pos:>7}/{len:7}")
            .progress_chars("#>-")
        );
        progress.set_message(alg.get_name());
        let mut compression_rate_sum = 0.0;
        let mut decompression_rate_sum = 0.0;

        for _ in 0..args.iterations {
          progress.inc(1);
          let start = Instant::now();
          let compressed = alg.compress(&buffer);
          compression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
          ratio = buffer.len() as f64 / compressed.len() as f64;

          let start = Instant::now();
          alg.decompress(&compressed);
          decompression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
        }
        progress.finish_and_clear();
        println!("{} compression: ratio={:.2} rate={:.1} MBps", alg.get_name(), ratio, compression_rate_sum / args.iterations as f64);
        println!("{} decompression: rate={:.1} MBps", alg.get_name(), decompression_rate_sum / args.iterations as f64);
      }
    },
    Operation::Compress => {
      let compressed = match args.algorithm {
        Algorithm::All => Vec::new(),
        Algorithm::Brotli => Brotli::new().compress(&buffer),
        Algorithm::Gzip => Gzip::new().compress(&buffer),
        Algorithm::Lz4 => Lz4::new().compress(&buffer),
        Algorithm::Snappy => Snappy::new().compress(&buffer),
        Algorithm::Zstd =>  Zstd::new().compress(&buffer),
      };
      if args.base64 {
        b64_writer.write_all(&compressed).unwrap();
      } else {
        io::stdout().write_all(&compressed).unwrap();
      }
    },
    Operation::Decompress => {
      let decompressed = match args.algorithm {
        Algorithm::All => Vec::new(),
        Algorithm::Brotli => Brotli::new().decompress(&buffer),
        Algorithm::Gzip => Gzip::new().decompress(&buffer),
        Algorithm::Lz4 => Lz4::new().decompress(&buffer),
        Algorithm::Snappy => Snappy::new().decompress(&buffer),
        Algorithm::Zstd =>  Zstd::new().decompress(&buffer),
      };
      io::stdout().write_all(&decompressed).unwrap();
    },
  }
}
