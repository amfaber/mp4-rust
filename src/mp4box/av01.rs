#![allow(unused)]
use bitvec::{field::BitField, prelude::*};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;
use bitcursor::BitCursor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Av01Box {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,

    #[serde(with = "value_u32")]
    pub horizresolution: FixedPointU16,

    #[serde(with = "value_u32")]
    pub vertresolution: FixedPointU16,
    pub frame_count: u16,
    pub depth: u16,

    pub av1c: Av1CBox,
}

impl Default for Av01Box {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            width: 0,
            height: 0,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            av1c: Av1CBox::default(),
        }
    }
}

impl Av01Box {
    pub fn new(config: &Av1Config) -> Self {
        Self {
            data_reference_index: 1,
            width: config.width,
            height: config.height,
            horizresolution: FixedPointU16::new(0x48),
            vertresolution: FixedPointU16::new(0x48),
            frame_count: 1,
            depth: 0x0018,
            av1c: Av1CBox::new(&config.sequence_header),
        }
    }

    pub fn get_type(&self) -> BoxType {
        BoxType::Av01Box
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 8 + 70 + self.av1c.box_size()
    }
}

impl Mp4Box for Av01Box {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "data_reference_index={} width={} height={} frame_count={}",
            self.data_reference_index, self.width, self.height, self.frame_count
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Av01Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        todo!()
    //     let start = box_start(reader)?;

    //     reader.read_u32::<BigEndian>()?; // reserved
    //     reader.read_u16::<BigEndian>()?; // reserved
    //     let data_reference_index = reader.read_u16::<BigEndian>()?;

    //     reader.read_u32::<BigEndian>()?; // pre-defined, reserved
    //     reader.read_u64::<BigEndian>()?; // pre-defined
    //     reader.read_u32::<BigEndian>()?; // pre-defined
    //     let width = reader.read_u16::<BigEndian>()?;
    //     let height = reader.read_u16::<BigEndian>()?;
    //     let horizresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
    //     let vertresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
    //     reader.read_u32::<BigEndian>()?; // reserved
    //     let frame_count = reader.read_u16::<BigEndian>()?;
    //     skip_bytes(reader, 32)?; // compressorname
    //     let depth = reader.read_u16::<BigEndian>()?;
    //     reader.read_i16::<BigEndian>()?; // pre-defined

    //     let end = start + size;
    //     loop {
    //         let current = reader.stream_position()?;
    //         if current >= end {
    //             return Err(Error::InvalidData("av1c not found"));
    //         }
    //         let header = BoxHeader::read(reader)?;
    //         let BoxHeader { name, size: s } = header;
    //         if s > size {
    //             return Err(Error::InvalidData(
    //                 "av01 box contains a box with a larger size than it",
    //             ));
    //         }
    //         if name == BoxType::Av1CBox {
    //             let av1c = Av1CBox::read_box(reader, s)?;

    //             skip_bytes_to(reader, start + size)?;

    //             return Ok(Self {
    //                 data_reference_index,
    //                 width,
    //                 height,
    //                 horizresolution,
    //                 vertresolution,
    //                 frame_count,
    //                 depth,
    //                 av1c,
    //             });
    //         } else {
    //             skip_bytes_to(reader, current + s)?;
    //         }
    //     }
    }
}

impl<W: Write> WriteBox<&mut W> for Av01Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined, reserved
        writer.write_u64::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u32::<BigEndian>(self.horizresolution.raw_value())?;
        writer.write_u32::<BigEndian>(self.vertresolution.raw_value())?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.frame_count)?;
        // skip compressorname
        write_zeros(writer, 32)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_i16::<BigEndian>(-1)?; // pre-defined

        self.av1c.write_box(writer)?;

        Ok(size)
    }
}

/// https://aomediacodec.github.io/av1-isobmff/ for reference
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Av1CBox {
    sequence_header: Vec<u8>,
}

impl Av1CBox {
    pub fn new(sequence_header: &[u8]) -> Self{
        Self{
            sequence_header: sequence_header.to_vec(),
        }
    }
    // pub fn new(mut config_obus: &[u8]) -> Result<Self> {
    //     let mut out = Self::default();
    //     let mut sequence_header_encountered = false;
    //     out.config_obus = config_obus.to_vec();
    //     loop {
    //         let (mut remaining, header) = ObuHeader::parse(config_obus)?;
    //         dbg!(remaining);
    //         dbg!(&header);
    //         if let Some(size) = header.size {
    //             config_obus = &remaining[size as usize..];
    //             remaining = &remaining[..size as usize];
    //         } else {
    //             config_obus = remaining;
    //         }
    //         if header.ty == ObuType::SequenceHeader {
    //             out.parse_sequence_header(remaining);
    //             sequence_header_encountered = true;
    //         }

    //         if config_obus.is_empty() {
    //             break;
    //         }
    //         config_obus = remaining;
    //     }
    //     if !sequence_header_encountered {
    //         return Err(Error::InvalidData("No sequence header for AV1"));
    //     }
    //     todo!()
    // }

    // pub fn parse_sequence_header(&mut self, bytes: &[u8]) {
    //     let mut cursor = BitCursor::new(bytes.view_bits());

    //     self.seq_profile = cursor.load_u8(3);
    //     cursor.skip(1); // still picture
    //     let reduced_still_picture_header = cursor.next_flag();
    //     if reduced_still_picture_header {
    //         self.seq_level_idx_0 = cursor.load_u8(5);
    //         self.seq_tier_0 = false;
    //     } else {
    //         let mut buffer_delay_length_minus_1 = 0;
    //         let mut decoder_model_info_present_flag = false;
    //         let timing_info_present_flag = cursor.next_flag();
    //         if timing_info_present_flag {
    //             cursor.skip(32); // num_units_in_display_tick
    //             cursor.skip(32); // time_scale
    //             if cursor.next_flag() {
    //                 cursor.uvlc_skip();
    //             }
    //             // decoder_model_info_present_flag
    //             decoder_model_info_present_flag = cursor.next_flag();
    //             if decoder_model_info_present_flag {
    //                 buffer_delay_length_minus_1 = cursor.load_u8(5);
    //                 cursor.skip(42);
    //             }
    //         }
    //         let initial_display_delay_present_flag = cursor.next_flag();
    //         let operating_points_cnt_minus_1 = cursor.load_u8(5);
    //         for i in 0..=operating_points_cnt_minus_1 {
    //             cursor.skip(12);
    //             let seq_level_idx = cursor.load_u8(5);
    //             let seq_tier = if seq_level_idx > 7 {
    //                 cursor.next_flag()
    //             } else {
    //                 false
    //             };
    //             if decoder_model_info_present_flag {
    //                 let decoder_model_present_for_this_op = cursor.next_flag();
    //                 if decoder_model_present_for_this_op {
    //                     cursor.skip((buffer_delay_length_minus_1 + 1) as usize); // decoder_buffer_delay
    //                     cursor.skip((buffer_delay_length_minus_1 + 1) as usize); // encoder_buffer_delay
    //                     cursor.skip(1) // low_delay_mode_flag
    //                 }
    //             }
    //             if initial_display_delay_present_flag {
    //                 let initial_display_delay_present_for_this_op = cursor.next_flag();
    //                 if initial_display_delay_present_for_this_op {
    //                     cursor.skip(4);
    //                 }
    //             }
    //             if i == 0 {
    //                 self.seq_tier_0 = seq_tier;
    //                 self.seq_level_idx_0 = seq_level_idx;
    //             }
    //         }
    //     }
    //     let frame_width_bits_minus_1 = cursor.load_u8(4);
    //     let frame_height_bits_minus_1 = cursor.load_u8(4);
    //     cursor.skip(frame_width_bits_minus_1 as usize + 1);
    //     cursor.skip(frame_height_bits_minus_1 as usize + 1);

    //     if !reduced_still_picture_header {
    //         let frame_id_numbers_present_flag = cursor.next_flag();
    //         if frame_id_numbers_present_flag {
    //             cursor.skip(7); // delta_frame_id_length_minus_2 (4), additional_frame_id_length_minus_1 (3)
    //         }
    //     }

    //     cursor.skip(3); // use_128x128_superblock (1), enable_filter_intra (1), enable_intra_edge_filter (1)

    //     if !reduced_still_picture_header {
    //         cursor.skip(4); // enable_interintra_compound (1), enable_masked_compound (1)
    //                         // enable_warped_motion (1), enable_dual_filter (1)
    //         let enable_order_hint = cursor.next_flag();
    //         if enable_order_hint {
    //             cursor.skip(2); // enable_jnt_comp (1), enable_ref_frame_mvs (1)
    //         }
    //         let seq_choose_screen_content_tools = cursor.next_flag();
    //         let seq_force_screen_content_tools = if seq_choose_screen_content_tools{
    //             2
    //         } else {
    //             cursor.load_u8(1)
    //         };
    //         if seq_force_screen_content_tools != 0{
    //             let seq_choose_integer_mv = cursor.next_flag();
    //             if !seq_choose_integer_mv{
    //                 cursor.skip(1); // seq_force_integer_mv
    //             }
    //         }
    //         if enable_order_hint{
    //             cursor.skip(3); // order_hint_bits_minus_1
    //         }
    //     }
    //     cursor.skip(3); // enable_superres (1), enable_cdef (1), enable_restoration (1)

    //     todo!("parse color config");

    //     cursor.skip(1); // film_grain_params_present
        
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ObuHeader {
    ty: ObuType,
    size: Option<u64>,
    temporal_id: u8,
    spatial_id: u8,
}

impl ObuHeader {
    /// Returns the remaining buffer and the parsed ObuHeader
    fn parse(bytes: &[u8]) -> Result<(&[u8], Self)> {
        let mut cursor = BitCursor::new(bytes.view_bits());
        let forbidden = cursor.next_flag();
        if forbidden {
            return Err(Error::InvalidData("Forbidden bit is set in OBU Header"));
        }

        let ty = ObuType::from_u8(cursor.load_u8(4))?;
        let extension_flag = cursor.next_flag();
        let has_size = cursor.next_flag();
        cursor.skip(1); // reserved
        let (temporal_id, spatial_id) = if extension_flag {
            (cursor.load_u8(3), cursor.load_u8(2))
        } else {
            (0, 0)
        };

        let size = has_size.then(|| cursor.leb128());
        dbg!(cursor.cursor_position());
        let remaining_buffer = &bytes[(cursor.cursor_position() / 8)..];
        let out = Self {
            ty,
            size,
            temporal_id,
            spatial_id,
        };
        Ok((remaining_buffer, out))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[repr(u8)]
enum ObuType {
    SequenceHeader = 1,
    TemporalDelimiter = 2,
    FrameHeader = 3,
    TileGroup = 4,
    MetaData = 5,
    Frame = 6,
    RedundantFrameHeader = 7,
    TileList = 8,
    Padding = 15,
}

impl ObuType {
    fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::SequenceHeader),
            2 => Ok(Self::TemporalDelimiter),
            3 => Ok(Self::FrameHeader),
            4 => Ok(Self::TileGroup),
            5 => Ok(Self::MetaData),
            6 => Ok(Self::Frame),
            7 => Ok(Self::RedundantFrameHeader),
            8 => Ok(Self::TileList),
            15 => Ok(Self::Padding),
            _ => Err(Error::InvalidData("Unregognized OBU Type")),
        }
    }
}

impl Mp4Box for Av1CBox {
    fn box_type(&self) -> BoxType {
        BoxType::Av1CBox
    }

    fn box_size(&self) -> u64 {
        HEADER_SIZE + self.sequence_header.len() as u64
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("Av1C is currently opaque"))
    }
}

type AV1CodecConfigurationRecord = BitArray<[u8; 5], Msb0>;

// impl<R: Read + Seek> ReadBox<&mut R> for Av1CBox {
//     fn read_box(reader: &mut R, size: u64) -> Result<Self> {
//         let start = box_start(reader)?;
//         let mut bits = AV1CodecConfigurationRecord::ZERO;
//         let mut cursor = BitCursor::new(&mut bits);
//         cursor
//             .next_mut(32)
//             .store_be(reader.read_u32::<BigEndian>()?);
//         cursor.next_mut(8).store_be(reader.read_u8()?);
//         cursor.seek(0);
//         let marker = cursor.next_flag();
//         if !marker {
//             return Err(Error::InvalidData("Marker isn't bit isn't 1 in Av1CBox"));
//         }

//         let end = start + size;
//         let mut config_obus = vec![0; (end - reader.stream_position()?) as usize];
//         reader.read_exact(&mut config_obus)?;

//         Ok(Self {
//             version: cursor.next(7).load_be(),
//             seq_profile: cursor.next(3).load_be(),
//             seq_level_idx_0: cursor.next(5).load_be(),
//             seq_tier_0: cursor.next_flag(),
//             high_bitdepth: cursor.next_flag(),
//             twelve_bit: cursor.next_flag(),
//             monochrome: cursor.next_flag(),
//             chroma_subsampling_x: cursor.next_flag(),
//             chroma_subsampling_y: cursor.next_flag(),
//             chroma_sample_position: cursor.next(2).load_be(),
//             config_obus,
//         })
//     }
// }

impl<W: Write> WriteBox<&mut W> for Av1CBox{
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;
        writer.write(&self.sequence_header)?;
        Ok(size)
    }
}

// impl<W: Write> WriteBox<&mut W> for Av1CBox {
//     fn write_box(&self, writer: &mut W) -> Result<u64> {
//         let mut bits = AV1CodecConfigurationRecord::ZERO;
//         let mut cursor = BitCursor::new(&mut bits);
//         cursor.next_mut(1).store_be(1); // marker is always = 1, makes it distinct from OBU Header
//         cursor.next_mut(7).store_be(self.version);
//         cursor.next_mut(3).store_be(self.seq_profile);
//         cursor.next_mut(5).store_be(self.seq_level_idx_0);
//         cursor.next_mut(1).store_be(self.seq_tier_0 as u8);
//         cursor.next_mut(1).store_be(self.high_bitdepth as u8);
//         cursor.next_mut(1).store_be(self.twelve_bit as u8);
//         cursor.next_mut(1).store_be(self.monochrome as u8);
//         cursor.next_mut(1).store_be(self.chroma_subsampling_x as u8);
//         cursor.next_mut(1).store_be(self.chroma_subsampling_y as u8);
//         cursor.next_mut(2).store_be(self.chroma_sample_position);
//         cursor.next_mut(8).store_be(0);
//         writer.write(&bits.into_inner())?;
//         let size = self.box_size();
//         Ok(size)
//     }
// }

mod bitcursor {
    use bitvec::prelude::*;
    use std::marker::PhantomData;
    pub struct BitCursor<S, T> {
        bitslice: S,
        cursor: usize,
        _phan: PhantomData<T>,
    }

    impl<S, T> BitCursor<S, T>
    where
        T: BitStore,
        S: AsRef<BitSlice<T, Msb0>>,
    {
        pub fn new(bitslice: S) -> Self {
            Self {
                bitslice,
                cursor: 0,
                _phan: PhantomData,
            }
        }

        pub fn cursor_position(&self) -> usize {
            self.cursor
        }

        pub fn next(&mut self, n: usize) -> &BitSlice<T, Msb0> {
            let cur = self.cursor;
            self.cursor += n;
            &self.bitslice.as_ref()[cur..self.cursor]
        }
        pub fn skip(&mut self, n: usize) {
            self.cursor += n;
        }
        pub fn load_u8(&mut self, n: usize) -> u8 {
            self.next(n).load_be()
        }
        pub fn next_flag(&mut self) -> bool {
            self.load_u8(1) == 1
        }

        pub fn seek(&mut self, position: usize) {
            self.cursor = position
        }

        pub fn uvlc_skip(&mut self) {
            let n = self.bitslice.as_ref()[self.cursor..].leading_zeros();
            self.skip(2 * n);
        }

        pub fn leb128(&mut self) -> u64 {
            let mut out = 0;

            for i in 0..8 {
                let byte = self.load_u8(8);
                out |= ((byte & 0x7f) as u64) << (i * 7);
                if (byte & 0x80) == 0 {
                    break;
                }
            }
            out
        }
    }

    impl<S, T> BitCursor<S, T>
    where
        T: BitStore,
        S: AsMut<BitSlice<T, Msb0>>,
    {
        pub fn next_mut(&mut self, n: usize) -> &mut BitSlice<T, Msb0> {
            let cur = self.cursor;
            self.cursor += n;
            &mut self.bitslice.as_mut()[cur..self.cursor]
        }
    }
}
