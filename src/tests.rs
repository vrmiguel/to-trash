use rand::{prelude::SmallRng, RngCore, SeedableRng};

pub fn dummy_bytes() -> Vec<u8> {
    let mut rng = SmallRng::from_entropy();
    let quantity = 1024 + rng.next_u32() % 1024;
    let mut vec = vec![0; quantity as usize];
    rng.fill_bytes(&mut vec);
    vec
}
