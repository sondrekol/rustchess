




pub fn pop_LSB(long:&mut u64) -> usize{

    let result = u64::trailing_zeros(long);

    for i in 0..64{
        if *long & 0x1 << i != 0{
            *long ^= 0x1 << i;
            return i;
        }
        
    }
    return 100;
}

pub fn seperate_bits(long:&u64) -> Vec<u64>{
    let mut result = Vec::<u64>::new();
    for i in 0..64{
        if *long & 0x1 << i != 0{
            result.push(*long & (0x1 << i));
        }
    }
    return result;
}

