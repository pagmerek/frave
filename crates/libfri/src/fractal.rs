use num::complex::Complex;

//fn get_literals<const N: usize>(d: f32) -> [Complex<f32>; N] {
//    let base = Complex::new(d / 2., (2. - (d / 2.).powf(2.)).sqrt());
//    let mut powers = [base; N];
//    let mut i = 1;
//
//    let mut pow = Complex::<f32> { re: 1., im: 0. };
//    while i < N {
//        powers[i] = Complex::<f32> {
//            re: (-pow.re / base.re).round(),
//            im: (pow.im / base.im).round(),
//        };
//        pow *= base;
//        i += 1;
//    }
//
//    powers[0] = Complex::<f32> { re: 0., im: 1. };
//
//    powers.swap(1, 2);
//
//    powers
//}



/*
 * CENTERS represent the optimal center of fractal space for biggest rectangle that has specific dimensions
 * Calculation can be found in fractal_lattice repo. This is statically encoded in codec
 * to save it from doing costly calculations in runtime. Unfortunately Rust const functions are
 * not yet sophisticated enough to calculate these numbers in compile time
 */
pub static CENTERS: [((i32, i32), Complex<i32>, u8); 15] = [
    ((17,8), Complex { re: 1, im: 2}, 9),
    ((47,9), Complex { re: 31, im: 1}, 10),
    ((41,26), Complex { re: 26, im: 18}, 11),
    ((88,15), Complex { re: 21, im: 7}, 12),
    ((108,65), Complex { re: 88, im: 43}, 14),
    ((227,60), Complex { re: 82, im: 41}, 15),
    ((202,149), Complex { re: 88, im: 40}, 16),
    ((284,84), Complex { re: 266, im: 52}, 17),
    ((649,148), Complex { re: 246, im: 130}, 18),
    ((651,418), Complex { re: 175, im: 130}, 19),
    ((1542, 333), Complex { re: 1120, im: 130}, 20),
    ((997,458), Complex { re: 449, im: 421}, 21),
    ((1148,883), Complex { re: 74, im: 320}, 22),
    ((4243,960), Complex { re: 2869, im: 215}, 23),
    ((3648,2439), Complex { re: 2375, im: 1725}, 24),
];

pub static LITERALS: [Complex<i32>; 30] = 
             [
               Complex { re: 0, im: 1 },
               Complex { re: -1, im: 1 },
               Complex { re: 2, im: 0 },
               Complex { re: -3, im: -1 },
               Complex { re: 5, im: -1 },
               Complex { re: 1, im: 3 },
               Complex { re: -11, im: -1 },
               Complex { re: 9, im: -5 },
               Complex { re: 13, im: 7 },
               Complex { re: -31, im: 3 },
               Complex { re: 5, im: -17 },
               Complex { re: 57, im: 11 },
               Complex { re: -67, im: 23 },
               Complex { re: -47, im: -45 },
               Complex { re: 181, im: -1 },
               Complex { re: -87, im: 91 },
               Complex { re: -275, im: -89 },
               Complex { re: 449, im: -93 },
               Complex { re: 101, im: 271 },
               Complex { re: -999, im: -85 },
               Complex { re: 797, im: -457 },
               Complex { re: 1201, im: 627 },
               Complex { re: -2795, im: 287 },
               Complex { re: 393, im: -1541 },
               Complex { re: 5197, im: 967 },
               Complex { re: -5983, im: 2115 },
               Complex { re: -4411, im: -4049 },
               Complex { re: 16377, im: -181 },
               Complex { re: -7555, im: 8279 },
               Complex {
                    re: -25199,
                    im: -7917,
                },
            ];

