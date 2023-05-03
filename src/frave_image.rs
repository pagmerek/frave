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

// pub fn get_quantization_matrix() -> Vec<f64> {
//  vec![
//             1.0,               // 0 
//             2.0f64.sqrt().sqrt(),     // 1
//             4.0f64.sqrt().sqrt(),     // 2
//             8.0f64.sqrt().sqrt(),     // 3
//             16.0f64.sqrt().sqrt(),    // 4
//             16.0f64.sqrt().sqrt(),    // 5
//             16.0f64.sqrt().sqrt(),    // 6
//             16.0f64.sqrt().sqrt(),   // 7
//             16.0f64.sqrt().sqrt(),   // 8
//             32.0f64.sqrt().sqrt(),   // 9
//             32.0f64.sqrt().sqrt(),   // 10
//             32.0f64.sqrt().sqrt(),   // 11
//             32.0f64.sqrt().sqrt(),   // 12
//             64.0f64.sqrt().sqrt(),   // 13
//             64.0f64.sqrt().sqrt(),   // 14
//             64.0f64.sqrt().sqrt(),   // 15
//             128.0f64.sqrt().sqrt(),   // 16
//             128.0f64.sqrt().sqrt(),   // 17
//             128.0f64.sqrt().sqrt(),   // 18
//             128.0f64.sqrt().sqrt()   // 19
//         ]
// }

pub fn get_quantization_matrix() -> Vec<f64> {
 vec![
            1.0,               // 0 
            2.0f64,     // 1
            2.0f64,     // 2
            2.0f64,     // 3
            2.0f64,    // 4
            2.0f64,    // 5
            2.0f64,    // 6
            2.0f64,   // 7
            2.0f64,   // 8
            2.0f64,   // 9
            2.0f64,   // 10
            2.0f64,   // 11
            2.0f64,   // 12
            2.0f64,   // 13
            2.0f64,   // 14
            2.0f64,   // 15
            4.0f64,   // 16
            5.0f64,   // 17
            6.0f64,   // 18
            2.0f64   // 19
        ]
}
