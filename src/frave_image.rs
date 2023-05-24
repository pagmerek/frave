use crate::utils::ans::AnsContext;
use crate::variants::Variant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FraveImage {
    pub height: u32,
    pub width: u32,
    pub depth: usize,
    pub center: (i32, i32),
    pub ans_contexts: Vec<AnsContext>,
    pub variant: Variant,
    #[serde(with = "serde_bytes")]
    pub compressed_coef: Vec<u8>,
}

pub fn get_quantization_matrix_soft() -> Vec<i32> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 4, 2]
}

pub fn get_quantization_matrix() -> Vec<i32> {
    vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 5, 9, 2]
}
