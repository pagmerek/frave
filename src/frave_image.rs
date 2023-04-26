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

pub fn get_quantization_matrix() -> Vec<f64> {
 vec![
            1.0,               // 0 
            2.0f64.sqrt().sqrt(),     // 1
            4.0f64.sqrt().sqrt(),     // 2
            8.0f64.sqrt().sqrt(),     // 3
            16.0f64.sqrt().sqrt(),    // 4
            32.0f64.sqrt().sqrt(),    // 5
            64.0f64.sqrt().sqrt(),    // 6
            128.0f64.sqrt().sqrt(),   // 7
            256.0f64.sqrt().sqrt(),   // 8
            512.0f64.sqrt().sqrt(),   // 9
            1024.0f64.sqrt().sqrt(),  // 10
            2048.0f64.sqrt().sqrt(),  // 11
            4096.0f64.sqrt().sqrt(),  // 12
            65536.0f64.sqrt().sqrt(),  // 13
            131072.0f64.sqrt().sqrt(), // 14
            131072.0f64.sqrt().sqrt(), // 15
            65536.0f64.sqrt().sqrt(), // 16
            131072.0f64.sqrt().sqrt(), // 17
            262144.0f64.sqrt().sqrt(), // 18
            520000.0f64.sqrt().sqrt(), // 18
            10520000.0f64.sqrt().sqrt(), // 18
        ]
}
