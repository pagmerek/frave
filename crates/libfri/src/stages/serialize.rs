use itertools::Itertools;
use num::traits::ToBytes;
use std::array::TryFromSliceError;
use std::error::Error;
use std::fmt::Display;
use std::mem;

use crate::images::{ColorSpace, FractalVariant, CompressedImage, ImageMetadata};
use crate::stages::entropy_coding::AnsContext;

#[derive(Debug)]
pub enum SerializeError {
    InvalidSignature,
    InvalidMetadata,
    MalformedImageBytes,
    SliceConversion(TryFromSliceError)
}

impl Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SerializeError::*;
        let var_name = match self {
            InvalidSignature => write!(f, "Invalid signature for FRIF image."),
            SliceConversion(e) => write!(f, "Slice from input bytes is malformed: {}", e),
            InvalidMetadata => write!(f, "Invalid metadata"),
            MalformedImageBytes => write!(f, "Malformed image bytes"),
        };
        var_name
    }
}

impl From<TryFromSliceError> for SerializeError {
    fn from(err: TryFromSliceError) -> Self {
        SerializeError::SliceConversion(err)
    }
}
impl Error for SerializeError {}

#[allow(non_snake_case, non_upper_case_globals)]
mod Segments {
    pub const EHD: &[u8] = &[0xFF, 0xB2]; // Entropy Header Data
    pub const DAT: &[u8] = &[0xFF, 0xB4]; // Data
    pub const EOC: &[u8] = &[0xFF, 0xB8]; // End Of Channel
    pub const EOI: &[u8] = &[0xFF, 0xDF]; // End Of Image
}

pub fn encode(mut image: CompressedImage) -> Result<Vec<u8>, SerializeError> {
    let mut serial = Vec::new();
    serial.reserve(mem::size_of_val(&image));

    serial.extend_from_slice(b"frif");
    serial.extend_from_slice(&image.metadata.height.to_le_bytes());
    serial.extend_from_slice(&image.metadata.width.to_le_bytes());

    let mut mdat: u32 = 0x0;

    // colorspace
    let colorspace = &image.metadata.colorspace.get_encoding();
    mdat |= colorspace << 30;

    // variant
    let variant = &image.metadata.variant.get_encoding();
    mdat |= variant << 28;

    serial.extend_from_slice(&mdat.to_le_bytes());

    let mut i = 0;
    while let Some((ctxs, encoded_bytes)) = &image.channel_data[i].take() {
        i += 1;
        for ctx in ctxs {
            serial.extend_from_slice(Segments::EHD);
            serial.extend_from_slice(
                &(ctx.symbols.len() * mem::size_of_val(&ctx.symbols[0])).to_le_bytes(),
            );
            serial.extend_from_slice(
                &ctx.symbols
                    .iter()
                    .flat_map(|s| s.to_le_bytes())
                    .collect::<Vec<u8>>(),
            );
            serial.extend_from_slice(
                &ctx.freqs
                    .iter()
                    .flat_map(|s| s.to_le_bytes())
                    .collect::<Vec<u8>>(),
            );
        }
        serial.extend_from_slice(Segments::DAT);
        serial.extend_from_slice(&encoded_bytes.len().to_le_bytes());
        serial.extend(encoded_bytes);
        serial.extend_from_slice(Segments::EOC);
        if i >= image.metadata.colorspace.num_channels() {
            break;
        }
    }

    serial.extend_from_slice(Segments::EOI);
    return Ok(serial);
}

type ChannelData = [Option<(Vec<AnsContext>, Vec<u8>)>; 3];

pub fn decode(bytes: Vec<u8>) -> Result<CompressedImage, SerializeError> {
    let mut offset = 0;
    if &bytes[offset..offset + 4] != b"frif" {
        return Err(SerializeError::InvalidSignature);
    }
    offset += 4;

    let height = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
    offset += 4;

    let width = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
    offset += 4;

    let metadata = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
    offset += 4;

    let colorspace = ColorSpace::from_encoding((metadata >> 30 & 0b11) as u8)?; 
    let variant = FractalVariant::from_encoding((metadata >> 28 & 0b11) as u8)?;

    let channel_data = deserialize_channel_data(&bytes, offset)?;

    Ok(CompressedImage {
        metadata: ImageMetadata {
            height,
            width,
            colorspace,
            variant,
        },
        channel_data,
    })
}

fn deserialize_channel_data(bytes: &Vec<u8>, mut offset: usize) -> Result<ChannelData, SerializeError> {
    let mut channel_data: ChannelData = [None, None, None];
    let mut ans_contexts: Vec<AnsContext> = vec![];
    let mut encoded_bytes: Vec<u8> = vec![];
    let mut i = 0;
    loop {
        match &bytes[offset..offset + 2] {
            Segments::EHD => {
                offset += 2;

                let hist_len = u64::from_le_bytes(bytes[offset..offset + 8].try_into()?) as usize;
                offset += 8;

                let symbols = bytes[offset..offset + hist_len]
                    .chunks_exact(4)
                    .map(|e| u32::from_le_bytes(e.try_into().unwrap()))
                    .collect();
                offset += hist_len;

                let freqs = bytes[offset..offset + hist_len]
                    .chunks_exact(4)
                    .map(|e| u32::from_le_bytes(e.try_into().unwrap()))
                    .collect();
                offset += hist_len;

                ans_contexts.push(AnsContext { symbols, freqs })
            }
            Segments::DAT => {
                offset += 2;

                let data_len =
                    u64::from_le_bytes(bytes[offset..offset + 8].try_into()?) as usize;
                offset += 8;

                let data = bytes[offset..offset + data_len as usize].to_vec();
                offset += data_len;

                encoded_bytes = data;
            }
            Segments::EOC => {
                offset += 2;

                channel_data[i] = Some((ans_contexts, encoded_bytes));
                ans_contexts = vec![];
                encoded_bytes = vec![];
                i += 1;
            }
            Segments::EOI => return Ok(channel_data),
            _other => return Err(SerializeError::MalformedImageBytes),
        }
    } 
}

