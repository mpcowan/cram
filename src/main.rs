use std::time::Duration;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::time::Instant;

use clap::Parser;

#[derive(clap::ArgEnum, Clone, Debug)]
enum Algorithm {
  All,
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

fn compress_lz4(input: &Vec<u8>) -> Vec<u8> {
  lz4_flex::compress_prepend_size(input)
}

fn decompress_lz4(input: &Vec<u8>) -> Vec<u8> {
  lz4_flex::decompress_size_prepended(&input).unwrap()
}

fn compress_snappy(input: &Vec<u8>) -> Vec<u8> {
  let snappy_writer = Vec::new();
  let mut snap_writer = snap::write::FrameEncoder::new(snappy_writer);
  io::copy(&mut &input[..], &mut snap_writer).unwrap();
  snap_writer.into_inner().unwrap()
}

fn decompress_snappy(input: &Vec<u8>) -> Vec<u8> {
  let mut rdr = snap::read::FrameDecoder::new(&input[..]);
  let mut snap_writer = Vec::new();
  io::copy(&mut rdr, &mut snap_writer).unwrap();
  snap_writer
}

fn compress_zstd(input: &Vec<u8>) -> Vec<u8> {
  let zstd_writer = Vec::new();
  let mut encoder = zstd::stream::Encoder::new(zstd_writer, 0).unwrap();
  io::copy(&mut &input[..], &mut encoder).unwrap();
  encoder.finish().unwrap()
}

fn decompress_zstd(input: &Vec<u8>) -> Vec<u8> {
  let mut zstd_writer = Vec::new();
  zstd::stream::copy_decode(&input[..], &mut zstd_writer).unwrap();
  zstd_writer
}

fn main() {
  let args = Args::parse();

  let mut f = File::open(args.file).unwrap();
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer).unwrap();

  match args.operation {
    Operation::Benchmark => {
      let size_in_mb = buffer.len() as f64 / 1048576 as f64;
      let mut compression_rate_sum = 0.0;
      let mut decompression_rate_sum = 0.0;
      let mut ratio = 100.0;

      for _ in 0..25 {
        let start = Instant::now();
        let compressed = compress_lz4(&buffer);
        compression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
        ratio = 100.0 * compressed.len() as f64 / buffer.len() as f64;

        let start = Instant::now();
        decompress_lz4(&compressed);
        decompression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
      }
      println!("\nlz4 compression: ratio={:.2}% rate={:.1} MBps", ratio, compression_rate_sum / 25.0);
      println!("lz4 decompression: rate={:.1} MBps", decompression_rate_sum / 25.0);

      compression_rate_sum = 0.0;
      decompression_rate_sum = 0.0;
      for _ in 0..25 {
        let start = Instant::now();
        let compressed = compress_snappy(&buffer);
        compression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
        ratio = 100.0 * compressed.len() as f64 / buffer.len() as f64;

        let start = Instant::now();
        decompress_snappy(&compressed);
        decompression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
      }
      println!("\nsnappy compression: ratio={:.2}% rate={:.1} MBps", ratio, compression_rate_sum / 25.0);
      println!("snappy decompression: rate={:.1} MBps", decompression_rate_sum / 25.0);

      compression_rate_sum = 0.0;
      decompression_rate_sum = 0.0;
      for _ in 0..25 {
        let start = Instant::now();
        let compressed = compress_zstd(&buffer);
        compression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
        ratio = 100.0 * compressed.len() as f64 / buffer.len() as f64;

        let start = Instant::now();
        decompress_zstd(&compressed);
        decompression_rate_sum += size_in_mb / elapsed_secs(start.elapsed());
      }
      println!("\nzstd compression: ratio={:.2}% rate={:.1} MBps", ratio, compression_rate_sum / 25.0);
      println!("zstd decompression: rate={:.1} MBps", decompression_rate_sum / 25.0);
    },
    Operation::Compress => {
      let compressed = match args.algorithm {
        Algorithm::All => Vec::new(),
        Algorithm::Lz4 => compress_lz4(&buffer),
        Algorithm::Snappy => compress_snappy(&buffer),
        Algorithm::Zstd => compress_zstd(&buffer),
      };
      io::stdout().write_all(&compressed).unwrap();
    },
    Operation::Decompress => {
      let decompressed = match args.algorithm {
        Algorithm::All => Vec::new(),
        Algorithm::Lz4 => decompress_lz4(&buffer),
        Algorithm::Snappy => decompress_snappy(&buffer),
        Algorithm::Zstd => decompress_zstd(&buffer),
      };
      io::stdout().write_all(&decompressed).unwrap();
    },
  }
}
