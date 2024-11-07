use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use std::{
    default,
    fs::File,
    io::{Cursor, Read, Seek, Write},
    str,
};

const C_RTFILE_TEXTURE_HEADER: &str = "RTTXTR";
const C_RTFILE_PACKAGE_LATEST_VERSION: u32 = 0;
const C_RTFILE_PACKAGE_HEADER: &str = "RTPACK";
const C_RTFILE_PACKAGE_HEADER_BYTE_SIZE: usize = 6;

const GL_UNSIGNED_BYTE: i32 = 0x1401;
const GL_UNSIGNED_SHORT_5_6_5: i32 = 0x8363;
const GL_UNSIGNED_SHORT_4_4_4_4: i32 = 0x8033;
const RT_FORMAT_EMBEDDED_FILE: i32 = 20000000;

#[derive(Debug, Default)]
struct RtFileHeader {
    file_type_id: [u8; C_RTFILE_PACKAGE_HEADER_BYTE_SIZE],
    version: u8,
    reseved: u8,
}

#[derive(Debug, Default)]
enum ECompressionType {
    #[default]
    None,
    Zlib,
}

#[derive(Debug, Default)]
enum ETextureFormat {
    #[default]
    GlUnsignedByte,
    GlUnsignedShort5_6_5,
    GlUnsignedShort4_4_4_4,
    RtFormatEmbeddedFile,
}

#[derive(Debug, Default)]
struct RtPackheader {
    file_header: RtFileHeader,
    compressed_size: u32,
    decompressed_size: u32,
    compression_type: ECompressionType,
    reserved: [u8; 15],
}

#[derive(Debug, Default)]
struct RttexHeader {
    file_header: RtFileHeader,
    height: i32,
    width: i32,
    format: ETextureFormat,
    original_height: i32,
    original_width: i32,
    b_uses_alpha: u8,
    b_already_compressed: u8,
    reseved_flags: [u8; 2],
    mip_map_count: i32,
    reserved: [u8; 16],
}

#[derive(Debug, Default)]
struct RttexMipHeader {
    height: i32,
    width: i32,
    data_size: i32,
    mip_level: i32,
    reserved: [u8; 2],
}

impl RtPackheader {
    fn deserialize(cursor: &mut Cursor<&Vec<u8>>) -> RtPackheader {
        let mut rt_pack_header = RtPackheader::default();

        cursor
            .read_exact(&mut rt_pack_header.file_header.file_type_id)
            .expect("Failed to read file type id");
        rt_pack_header.file_header.version = cursor.read_u8().expect("Failed to read version");
        rt_pack_header.file_header.reseved = cursor
            .read_u8()
            .expect("Failed to read file header reserved");
        rt_pack_header.compressed_size = cursor
            .read_u32::<LittleEndian>()
            .expect("Failed to read compressed size");
        rt_pack_header.decompressed_size = cursor
            .read_u32::<LittleEndian>()
            .expect("Failed to read decompressed size");
        rt_pack_header.compression_type =
            match cursor.read_u8().expect("Failed to read compression type") {
                0 => ECompressionType::None,
                1 => ECompressionType::Zlib,
                _ => panic!("Unknown compression type"),
            };
        cursor
            .read_exact(&mut rt_pack_header.reserved)
            .expect("Failed to read reserved");

        rt_pack_header
    }
}

impl RttexHeader {
    fn deserialize(cursor: &mut Cursor<Vec<u8>>) -> RttexHeader {
        let mut rttex_header = RttexHeader::default();

        cursor
            .read_exact(&mut rttex_header.file_header.file_type_id)
            .expect("Failed to read file type id");
        rttex_header.file_header.version = cursor.read_u8().expect("Failed to read version");
        rttex_header.file_header.reseved = cursor
            .read_u8()
            .expect("Failed to read file header reserved");
        rttex_header.height = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read height");
        rttex_header.width = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read width");
        rttex_header.format = match cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read format")
        {
            GL_UNSIGNED_BYTE => ETextureFormat::GlUnsignedByte,
            GL_UNSIGNED_SHORT_5_6_5 => ETextureFormat::GlUnsignedShort5_6_5,
            GL_UNSIGNED_SHORT_4_4_4_4 => ETextureFormat::GlUnsignedShort4_4_4_4,
            RT_FORMAT_EMBEDDED_FILE => ETextureFormat::RtFormatEmbeddedFile,
            _ => panic!("Unknown texture format"),
        };
        rttex_header.original_height = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read original height");
        rttex_header.original_width = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read original width");
        rttex_header.b_uses_alpha = cursor.read_u8().expect("Failed to read uses alpha");
        rttex_header.b_already_compressed =
            cursor.read_u8().expect("Failed to read already compressed");
        cursor
            .read_exact(&mut rttex_header.reseved_flags)
            .expect("Failed to read reserved flags");
        rttex_header.mip_map_count = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read mip map count");
        cursor
            .read_exact(&mut rttex_header.reserved)
            .expect("Failed to read reserved");

        rttex_header
    }
}

fn is_a_packed_file(data: &Vec<u8>) -> bool {
    str::from_utf8(&data[0..C_RTFILE_PACKAGE_HEADER_BYTE_SIZE])
        .expect("Failed to convert to string")
        == C_RTFILE_PACKAGE_HEADER
}

fn deflate_zlib(data: Vec<u8>) -> Vec<u8> {
    let mut e = ZlibDecoder::new(&data[..]);
    let mut buffer = Vec::new();
    e.read_to_end(&mut buffer).expect("Failed to deflate zlib");

    buffer
}

fn main() {
    let mut file_data = Vec::new();
    let file = File::open("tiles_page1.rttex")
        .expect("Failed to open file")
        .read_to_end(&mut file_data)
        .expect("Failed to read file");

    let is_a_packed_file = is_a_packed_file(&file_data);
    println!("Is a packed file: {}", is_a_packed_file);

    if is_a_packed_file {
        let mut cursor = Cursor::new(&file_data);
        let rt_pack_header = RtPackheader::deserialize(&mut cursor);
        let decompressed_data = deflate_zlib(file_data[cursor.position() as usize..].to_vec());
        let mut cursor = Cursor::new(decompressed_data);
        let rttex_header = RttexHeader::deserialize(&mut cursor);
        println!("{:?}", rttex_header);
    }
}
