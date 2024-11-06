use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{Cursor, Read, Seek},
};

#[derive(Debug, Default)]
struct RtFileHeader {
    file_type_id: [char; 6],
    version: u8,
    reseved: u8,
}

enum ECompressionType {
    None,
    Zlib,
}

struct RtPackheader {
    file_header: RtFileHeader,
    compressed_size: u32,
    decompressed_size: u32,
    compression_type: ECompressionType,
    reserved: [u8; 15],
}

struct RttexHeader {
    file_header: RtFileHeader,
    height: i32,
    width: i32,
    format: i32,
    original_height: i32,
    original_width: i32,
    b_uses_alpha: u8,
    b_already_compressed: u8,
    reseved_flags: [u8; 2],
    mip_map_count: i32,
    reserved: [u8; 16],
}

struct RttexMipHeader {
    height: i32,
    width: i32,
    data_size: i32,
    mip_level: i32,
    reserved: [u8; 2],
}

fn main() {}
