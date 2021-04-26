use fnv::FnvHasher;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Seed(u64);
impl Seed {
  const DELIM: u64 = 0xe16013eafc14eeed;
  pub fn new<H: Hash>(seed: H) -> Self {
    Self(0).fork(seed)
  }
  pub fn fork<H: Hash>(&self, route: H) -> Self {
    let mut h = FnvHasher::with_key(self.0);
    Self::DELIM.hash(&mut h);
    route.hash(&mut h);
    Self(h.finish())
  }
  pub fn rng(self) -> SmallRng {
    let mut h = FnvHasher::with_key(self.0);
    Self::DELIM.hash(&mut h);
    SmallRng::seed_from_u64(h.finish())
  }
}
