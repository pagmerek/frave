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
