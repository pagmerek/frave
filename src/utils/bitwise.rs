pub fn get_next_power_two(mut num: u32) -> u32 {
    if num == 0 {
        return 1;
    }
    num -= 1;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
    num += 1;
    num
}

pub fn get_prev_power_two(x: u32) -> u32 {
    let mut num = x;
    num |= num >> 1;
    num |= num >> 2;
    num |= num >> 4;
    num |= num >> 8;
    num |= num >> 16;
 
    return num ^ (num >> 1);
}
