use std::io;

pub trait Compressor {
  fn compress(&self, input: &[u8]) -> Vec<u8>;
  fn decompress(&self, input: &[u8]) -> Vec<u8>;
  fn get_name(&self) -> &'static str;
}

// BROTLI //

pub struct Brotli {
  pub name: &'static str,
}

impl Brotli {
  pub fn new() -> Brotli {
    Brotli { name: "brotli" }
  }
}

impl Compressor for Brotli {
  fn compress(&self, input: &[u8]) -> Vec<u8> {
    let target = Vec::new();
    let mut encoder = brotli::CompressorWriter::new(target, 4096, 0, 20);
    io::copy(&mut &*input, &mut encoder).unwrap();
    encoder.into_inner()
  }
  fn decompress(&self, input: &[u8]) -> Vec<u8> {
    let mut decoder = brotli::Decompressor::new(input, 4096);
    let mut target = Vec::new();
    io::copy(&mut decoder, &mut target).unwrap();
    target
  }
  fn get_name(&self) -> &'static str {
    self.name
  }
}

// GZIP //

pub struct Gzip {
  pub name: &'static str,
}

impl Gzip {
  pub fn new() -> Gzip {
    Gzip { name: "gzip" }
  }
}

impl Compressor for Gzip {
  fn compress(&self, input: &[u8]) -> Vec<u8> {
    let target = Vec::new();
    let mut encoder = libflate::gzip::Encoder::new(target).unwrap();
    io::copy(&mut &*input, &mut encoder).unwrap();
    encoder.finish().into_result().unwrap()
  }
  fn decompress(&self, input: &[u8]) -> Vec<u8> {
    let mut decoder = libflate::gzip::Decoder::new(input).unwrap();
    let mut target = Vec::new();
    io::copy(&mut decoder, &mut target).unwrap();
    target
  }
  fn get_name(&self) -> &'static str {
    self.name
  }
}

// LZ4 //

pub struct Lz4 {
  pub name: &'static str,
}

impl Lz4 {
  pub fn new() -> Lz4 {
    Lz4 { name: "lz4" }
  }
}

impl Compressor for Lz4 {
  fn compress(&self, input: &[u8]) -> Vec<u8> {
    lz4_flex::compress_prepend_size(input)
  }
  fn decompress(&self, input: &[u8]) -> Vec<u8> {
    lz4_flex::decompress_size_prepended(input).unwrap()
  }
  fn get_name(&self) -> &'static str {
    self.name
  }
}

// SNAPPY //

pub struct Snappy {
  pub name: &'static str,
}

impl Snappy {
  pub fn new() -> Snappy {
    Snappy { name: "snappy" }
  }
}

impl Compressor for Snappy {
  fn compress(&self, input: &[u8]) -> Vec<u8> {
    let target = Vec::new();
    let mut encoder = snap::write::FrameEncoder::new(target);
    io::copy(&mut &*input, &mut encoder).unwrap();
    encoder.into_inner().unwrap()
  }
  fn decompress(&self, input: &[u8]) -> Vec<u8> {
    let mut decoder = snap::read::FrameDecoder::new(input);
    let mut target = Vec::new();
    io::copy(&mut decoder, &mut target).unwrap();
    target
  }
  fn get_name(&self) -> &'static str {
    self.name
  }
}

// ZSTD //

pub struct Zstd {
  pub name: &'static str,
}

impl Zstd {
  pub fn new() -> Zstd {
    Zstd { name: "zstd" }
  }
}

impl Compressor for Zstd {
  fn compress(&self, input: &[u8]) -> Vec<u8> {
    let target = Vec::new();
    // while level 3 is the default, level 1 seems more fair for this comparison
    let mut encoder = zstd::stream::Encoder::new(target, 1).unwrap();
    io::copy(&mut &*input, &mut encoder).unwrap();
    encoder.finish().unwrap()
  }
  fn decompress(&self, input: &[u8]) -> Vec<u8> {
    let mut target = Vec::new();
    zstd::stream::copy_decode(input, &mut target).unwrap();
    target
  }
  fn get_name(&self) -> &'static str {
    self.name
  }
}
