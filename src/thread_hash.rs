use rand::{self, Rng};

#[allow(dead_code)]
pub struct ThreadHash {
    pub value: isize
}

#[allow(dead_code)]
impl ThreadHash {
    pub fn new() -> ThreadHash {
        let rn: isize = rand::thread_rng().gen::<isize>();
        ThreadHash { value: if rn == 0 { 1 } else { rn } }
    }

    #[inline(always)]
    pub fn rehash(prev: isize) -> ThreadHash {
        let mut nh = prev;
        nh ^= nh << 13;
        let mut u = nh as usize;
        u ^= u >> 17;
        let mut f = u as isize;
        f ^= f << 5;
        ThreadHash { value: f }
    }
}
