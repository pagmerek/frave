use crate::images::{FractalVariant, ColorSpace, CompressedImage, RasterImage, ImageMetadata};
use crate::stages::wavelet_transform::WaveletImage;
use crate::stages::{channel_transform, entropy_coding, quantization, serialize, wavelet_transform};

enum EncoderStage {
    RawImage(RasterImage),
    ChannelTransform(RasterImage),
    WaveletTransform(RasterImage),
    Quantization(WaveletImage),
    EntropyEncoding(WaveletImage),
    EncodedImage(CompressedImage),
    SerializedImage(Vec<u8>),
    Failure(String),
}

impl EncoderStage {
    fn forward(self) -> EncoderStage {
        match self {
            EncoderStage::RawImage(data) => EncoderStage::ChannelTransform(data),
            EncoderStage::ChannelTransform(data) => match channel_transform::encode(data) {
                Ok(result) => EncoderStage::WaveletTransform(result),
                Err(reason) => EncoderStage::Failure(reason),
            },
            EncoderStage::WaveletTransform(data) => match wavelet_transform::encode(data) {
                Ok(result) => EncoderStage::Quantization(result),
                Err(reason) => EncoderStage::Failure(reason),
            },
            EncoderStage::Quantization(data) => match quantization::encode(data) {
                Ok(result) => EncoderStage::EntropyEncoding(result),
                Err(reason) => EncoderStage::Failure(reason),
            },
            EncoderStage::EntropyEncoding(data) => match entropy_coding::encode(data) {
                Ok(result) => EncoderStage::EncodedImage(result),
                Err(reason) => EncoderStage::Failure(reason),
            },
            EncoderStage::EncodedImage(data) => match serialize::encode(data) {
                Ok(result) => EncoderStage::SerializedImage(result),
                Err(reason) => EncoderStage::Failure(reason.to_string())
            }
            other => other,
        }
    }
}

pub struct FRIEncoder {}

impl FRIEncoder {
    pub fn encode(
        self,
        data: Vec<u8>,
        height: u32,
        width: u32,
        colorspace: ColorSpace,
    ) -> Result<Vec<u8>, String> {
        let image = RasterImage {
            data,
            metadata: ImageMetadata{height, width, colorspace, variant: FractalVariant::TameTwindragon}
        };

        let mut stage = EncoderStage::RawImage(image);
        while !matches!(stage, EncoderStage::SerializedImage(_) | EncoderStage::Failure(_)) {
            stage = stage.forward();
        }

        match stage {
            EncoderStage::SerializedImage(image) => Ok(image),
            EncoderStage::Failure(msg) => Err(String::from("Failed to decode: ".to_owned() + &msg)),
            _ => unreachable!(),
        }
    }
}
