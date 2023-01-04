use crate::coord::Coord;
use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Variant {
    Boxes,
    Twindragon,
    TameTwindragon,
    SkewedTameTwindragon,
}

pub fn get_variant(var: Variant) -> [Coord; 30] {
    match var {
        Variant::Boxes => [
            Coord { x: 1, y: 0 },
            Coord { x: 0, y: 1 },
            Coord { x: -2, y: 0 },
            Coord { x: 0, y: -2 },
            Coord { x: 4, y: 0 },
            Coord { x: 0, y: 4 },
            Coord { x: -8, y: 0 },
            Coord { x: 0, y: -8 },
            Coord { x: 16, y: 0 },
            Coord { x: 0, y: 16 },
            Coord { x: -32, y: 0 },
            Coord { x: 0, y: -32 },
            Coord { x: 64, y: 0 },
            Coord { x: 0, y: 64 },
            Coord { x: -128, y: 0 },
            Coord { x: 0, y: -128 },
            Coord { x: 256, y: 0 },
            Coord { x: 0, y: 256 },
            Coord { x: -512, y: 0 },
            Coord { x: 0, y: -512 },
            Coord { x: 1024, y: 0 },
            Coord { x: 0, y: 1024 },
            Coord { x: -2048, y: 0 },
            Coord { x: 0, y: -2048 },
            Coord { x: 4096, y: 0 },
            Coord { x: 0, y: 4096 },
            Coord { x: -8192, y: 0 },
            Coord { x: 0, y: -8192 },
            Coord { x: 16384, y: 0 },
            Coord { x: 0, y: 16384 },
        ],
        Variant::Twindragon => [
            Coord { x: 1, y: 0 },
            Coord { x: -1, y: 1 },
            Coord { x: 0, y: -2 },
            Coord { x: 2, y: 2 },
            Coord { x: -4, y: 0 },
            Coord { x: 4, y: -4 },
            Coord { x: 0, y: 8 },
            Coord { x: -8, y: -8 },
            Coord { x: 16, y: 0 },
            Coord { x: -16, y: 16 },
            Coord { x: 0, y: -32 },
            Coord { x: 32, y: 32 },
            Coord { x: -64, y: 0 },
            Coord { x: 64, y: -64 },
            Coord { x: 0, y: 128 },
            Coord { x: -128, y: -128 },
            Coord { x: 256, y: 0 },
            Coord { x: -256, y: 256 },
            Coord { x: 0, y: -512 },
            Coord { x: 512, y: 512 },
            Coord { x: -1024, y: 0 },
            Coord { x: 1024, y: -1024 },
            Coord { x: 0, y: 2048 },
            Coord { x: -2048, y: -2048 },
            Coord { x: 4096, y: 0 },
            Coord { x: -4096, y: 4096 },
            Coord { x: 0, y: -8192 },
            Coord { x: 8192, y: 8192 },
            Coord { x: -16384, y: 0 },
            Coord {
                x: 16384,
                y: -16384,
            },
        ],
        Variant::TameTwindragon => [
            Coord { x: 0, y: 1 },
            Coord { x: -1, y: 1 },
            Coord { x: 2, y: 0 },
            Coord { x: -3, y: -1 },
            Coord { x: 5, y: -1 },
            Coord { x: 1, y: 3 },
            Coord { x: -11, y: -1 },
            Coord { x: 9, y: -5 },
            Coord { x: 13, y: 7 },
            Coord { x: -31, y: 3 },
            Coord { x: 5, y: -17 },
            Coord { x: 57, y: 11 },
            Coord { x: -67, y: 23 },
            Coord { x: -47, y: -45 },
            Coord { x: 181, y: -1 },
            Coord { x: -87, y: 91 },
            Coord { x: -275, y: -89 },
            Coord { x: 449, y: -93 },
            Coord { x: 101, y: 271 },
            Coord { x: -999, y: -85 },
            Coord { x: 797, y: -457 },
            Coord { x: 1201, y: 627 },
            Coord { x: -2795, y: 287 },
            Coord { x: 393, y: -1541 },
            Coord { x: 5197, y: 967 },
            Coord { x: -5983, y: 2115 },
            Coord { x: -4411, y: -4049 },
            Coord { x: 16377, y: -181 },
            Coord { x: -7555, y: 8279 },
            Coord {
                x: -25199,
                y: -7917,
            },
        ],
        Variant::SkewedTameTwindragon => [
            Coord { x: -1, y: 0 },
            Coord { x: 1, y: 1 },
            Coord { x: -1, y: -2 },
            Coord { x: 3, y: 2 },
            Coord { x: -1, y: 2 },
            Coord { x: -5, y: -6 },
            Coord { x: 7, y: 2 },
            Coord { x: 3, y: 10 },
            Coord { x: -17, y: -14 },
            Coord { x: 11, y: -6 },
            Coord { x: 23, y: 34 },
            Coord { x: -45, y: -22 },
            Coord { x: -1, y: -46 },
            Coord { x: 91, y: 90 },
            Coord { x: -89, y: 2 },
            Coord { x: -93, y: -182 },
            Coord { x: 271, y: 178 },
            Coord { x: -85, y: 186 },
            Coord { x: -457, y: -542 },
            Coord { x: 627, y: 170 },
            Coord { x: 287, y: 914 },
            Coord { x: -1541, y: -1254 },
            Coord { x: 967, y: -574 },
            Coord { x: 2115, y: 3082 },
            Coord { x: -4049, y: -1934 },
            Coord { x: -181, y: -4230 },
            Coord { x: 8279, y: 8098 },
            Coord { x: -7917, y: 362 },
            Coord {
                x: -8641,
                y: -16558,
            },
            Coord { x: 24475, y: 15834 },
        ],
    }
}
