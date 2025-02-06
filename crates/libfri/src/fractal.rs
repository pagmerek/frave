use num::complex::Complex;

fn get_literals<const N: usize>(d: f32) -> [Complex<f32>; N] {
    let base = Complex::new(d / 2., (2. - (d / 2.).powf(2.)).sqrt());
    let mut powers = [base; N];
    let mut i = 1;

    let mut pow = Complex::<f32> { re: 1., im: 0. };
    while i < N {
        powers[i] = Complex::<f32> {
            re: (-pow.re / base.re).round(),
            im: (pow.im / base.im).round(),
        };
        pow *= base;
        i += 1;
    }

    powers[0] = Complex::<f32> { re: 0., im: 1. };

    powers.swap(1, 2);

    powers
}

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

