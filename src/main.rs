use std::time::Duration;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::time::Instant;

use clap::Parser;

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
  operation: Operation
}

fn elapsed_secs(elapsed: Duration) -> f64 {
  elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
}

fn main() {
  let args = Args::parse();

  let mut f = File::open(args.file).unwrap();
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer).unwrap();

  match args.operation {
    Operation::Benchmark => {
      let size_in_mb = buffer.len() as f64 / 1048576 as f64;
      let mut ratio = 100.0;

      let mut algs: Vec<Box<dyn Compressor>> = Vec::new();
      algs.push(Box::new(Brotli::new()));
      algs.push(Box::new(Gzip::new()));
      algs.push(Box::new(Lz4::new()));
      algs.push(Box::new(Snappy::new()));
      algs.push(Box::new(Zstd::new()));

      for alg in algs {
        let mut compression_rate_sum = 0.0;
        let mut decompression_rate_sum = 0.0;

        for _ in 0..25 {
          let start = Instant::now();
          let compressed = alg.compress(&buffer);
          compression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
          ratio = buffer.len() as f64 / compressed.len() as f64;

          let start = Instant::now();
          alg.decompress(&compressed);
          decompression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
        }
        println!("\n{} compression: ratio={:.2} rate={:.1} MBps", alg.get_name(), ratio, compression_rate_sum / 25.0);
        println!("{} decompression: rate={:.1} MBps", alg.get_name(), decompression_rate_sum / 25.0);
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
      io::stdout().write_all(&compressed).unwrap();
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
