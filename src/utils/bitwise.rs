pub fn get_prev_power_two(x: u32) -> u32 {
    let mut num = x;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;

    num ^ (num >> 1)
}
