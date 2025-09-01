use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use image::{ImageBuffer, Rgba};
use std::{
    fs::File,
    io::{Cursor, Read},
    str,
};

const C_RTFILE_TEXTURE_HEADER: &str = "RTTXTR";
const C_RTFILE_PACKAGE_LATEST_VERSION: u8 = 0;
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
    reserved: u8,
}

#[derive(Debug, Default)]
enum ECompressionType {
    #[default]
    None,
    Zlib,
}

#[derive(Debug, Default)]
#[repr(i32)]
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

#[derive(Debug)]
struct RttexHeader {
    file_header: RtFileHeader,
    height: i32,
    width: i32,
    format: ETextureFormat,
    original_height: i32,
    original_width: i32,
    b_uses_alpha: u8,
    b_already_compressed: u8,
    reserved_flags: [u8; 2],
    mip_map_count: i32,
    reserved: [u8; 16 * 4],
}

#[derive(Debug, Default)]
struct RttexMipHeader {
    height: i32,
    width: i32,
    data_size: i32,
    mip_level: i32,
    reserved: [u8; 2 * 4],
}

impl RtPackheader {
    fn deserialize(cursor: &mut Cursor<&Vec<u8>>) -> RtPackheader {
        let mut rt_pack_header = RtPackheader::default();

        cursor
            .read_exact(&mut rt_pack_header.file_header.file_type_id)
            .expect("Failed to read file type id");
        rt_pack_header.file_header.version = cursor.read_u8().expect("Failed to read version");
        rt_pack_header.file_header.reserved = cursor
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

impl Default for RttexHeader {
    fn default() -> Self {
        RttexHeader {
            file_header: RtFileHeader::default(),
            height: 0,
            width: 0,
            format: ETextureFormat::GlUnsignedByte,
            original_height: 0,
            original_width: 0,
            b_uses_alpha: 0,
            b_already_compressed: 0,
            reserved_flags: [0; 2],
            mip_map_count: 0,
            reserved: [0; 64],
        }
    }
}

impl RttexHeader {
    fn deserialize(cursor: &mut Cursor<&Vec<u8>>) -> RttexHeader {
        let mut rttex_header = RttexHeader::default();

        cursor
            .read_exact(&mut rttex_header.file_header.file_type_id)
            .expect("Failed to read file type id");
        rttex_header.file_header.version = cursor.read_u8().expect("Failed to read version");
        rttex_header.file_header.reserved = cursor
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
            .read_exact(&mut rttex_header.reserved_flags)
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

impl RttexMipHeader {
    fn deserialize(cursor: &mut Cursor<&Vec<u8>>) -> RttexMipHeader {
        let mut rttex_mip_header = RttexMipHeader::default();

        rttex_mip_header.height = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read height");
        rttex_mip_header.width = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read width");
        rttex_mip_header.data_size = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read data size");
        rttex_mip_header.mip_level = cursor
            .read_i32::<LittleEndian>()
            .expect("Failed to read mip level");
        cursor
            .read_exact(&mut rttex_mip_header.reserved)
            .expect("Failed to read reserved");

        rttex_mip_header
    }
}

fn is_a_packed_file(data: &Vec<u8>) -> bool {
    if data.len() < C_RTFILE_PACKAGE_HEADER_BYTE_SIZE {
        return false;
    }
    str::from_utf8(&data[0..C_RTFILE_PACKAGE_HEADER_BYTE_SIZE])
        .expect("Failed to convert to string")
        == C_RTFILE_PACKAGE_HEADER
}

fn is_a_txtr_file(data: &Vec<u8>) -> bool {
    if data.len() < C_RTFILE_PACKAGE_HEADER_BYTE_SIZE {
        return false;
    }
    str::from_utf8(&data[0..C_RTFILE_PACKAGE_HEADER_BYTE_SIZE])
        .expect("Failed to convert to string")
        == C_RTFILE_TEXTURE_HEADER
}

fn deflate_zlib(data: Vec<u8>) -> Vec<u8> {
    let mut e = ZlibDecoder::new(&data[..]);
    let mut buffer = Vec::new();
    e.read_to_end(&mut buffer).expect("Failed to deflate zlib");

    buffer
}

pub fn get_image_buffer(path: &str) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let mut file_data = Vec::new();
    File::open(path)
        .expect("Failed to open file")
        .read_to_end(&mut file_data)
        .expect("Failed to read file");

    let is_a_packed_file = is_a_packed_file(&file_data);
    if is_a_packed_file {
        let mut cursor = Cursor::new(&file_data);
        let rt_pack_header = RtPackheader::deserialize(&mut cursor);

        if rt_pack_header.file_header.version != C_RTFILE_PACKAGE_LATEST_VERSION {
            panic!("Unsupported version");
        }

        let decompressed_data = deflate_zlib(file_data[cursor.position() as usize..].to_vec());
        
        if decompressed_data.is_empty() {
            return None;
        }
        
        let is_a_txtr_file = is_a_txtr_file(&decompressed_data);

        if is_a_txtr_file {
            let mut cursor = Cursor::new(&decompressed_data);
            let rttex_header = RttexHeader::deserialize(&mut cursor);

            for _ in 0..rttex_header.mip_map_count {
                let _ = RttexMipHeader::deserialize(&mut cursor);
            }

            let mut image_data = vec![0; (rttex_header.width * rttex_header.height * 4) as usize];
            cursor
                .read_exact(&mut image_data)
                .expect("Failed to read image data");

            let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                rttex_header.width as u32,
                rttex_header.height as u32,
                image_data,
            )
            .expect("Failed to create image buffer");

            let img = image::imageops::flip_horizontal(&img);
            let img = image::imageops::rotate180(&img);

            return Some(img);
        }
    }
    None
}
