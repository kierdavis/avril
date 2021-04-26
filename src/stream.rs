use itertools::Itertools;
use std::time::Duration;

pub struct Stream<'a, E: 'a>(Box<dyn Iterator<Item = (Duration, E)> + 'a>);

impl<'a, E: 'a> Stream<'a, E> {
  pub fn chain(self, other: Self) -> Self {
    Self::from_iter(self.into_iter().chain(other))
  }
  pub fn chain_at(self, threshold: Duration, other: Self) -> Self {
    Self::from_iter(ChainAt {
      source1: self.into_iter(),
      source2: other.into_iter(),
      threshold,
    })
  }
  pub fn coalesce<F>(mut self, reduce: F) -> Self
  where
    F: Fn(E, E) -> E + 'a,
  {
    Self::from_iter(Coalesce {
      head: self.next(),
      source: self.into_iter(),
      reduce,
    })
  }
  pub fn delay(self, duration: Duration) -> Self {
    Self::empty().chain_at(duration, self)
  }
  pub fn drop(self, duration: Duration) -> Self {
    Self::from_iter(Drop {
      source: self.into_iter(),
      duration,
    })
  }
  pub fn empty() -> Self {
    Self::from_iter(std::iter::empty())
  }
  pub fn flat_map<F, I>(self, fun: F) -> Stream<'a, I::Item>
  where
    F: FnMut(E) -> I + 'a,
    I: IntoIterator + 'a,
  {
    self.map(fun).flatten()
  }
  pub fn flatten(self) -> Stream<'a, E::Item>
  where
    E: IntoIterator,
  {
    Stream::from_iter(self.into_iter().flat_map(|(d, es)| {
      let ds = std::iter::once(d).chain(std::iter::repeat(Duration::from_secs(0)));
      ds.zip(es)
    }))
  }
  pub fn from_iter<I>(iter: I) -> Self
  where
    I: IntoIterator<Item = (Duration, E)>,
    I::IntoIter: 'a,
  {
    Self(Box::new(iter.into_iter()))
  }
  pub fn immediate(event: E) -> Self {
    Self::from_iter(std::iter::once((Duration::from_secs(0), event)))
  }
  pub fn lazy<F: 'a>(fun: F) -> Self
  where
    F: FnOnce() -> Self,
  {
    Self::from_iter(Lazy::Before(Some(fun)))
  }
  pub fn map<F, EE>(self, mut fun: F) -> Stream<'a, EE>
  where
    F: FnMut(E) -> EE + 'a,
  {
    Stream::from_iter(self.into_iter().map(move |(d, e)| (d, fun(e))))
  }
  pub fn merge(mut self, mut other: Self) -> Self {
    Self::from_iter(Merge {
      head1: self.next(),
      head2: other.next(),
      source1: self.into_iter(),
      source2: other.into_iter(),
    })
  }
  pub fn merge_all<I>(streams: I) -> Self
  where
    I: IntoIterator<Item = Self>,
  {
    streams
      .into_iter()
      .fold1(Self::merge)
      .unwrap_or_else(Self::empty)
  }
  pub fn next(&mut self) -> Option<(Duration, E)> {
    self.0.next()
  }
  pub fn repeat_every(self, interval: Duration) -> Self
  where
    E: Clone,
  {
    let sample: Vec<_> = self.take(interval).into_iter().collect();
    Self::replay_every(sample, interval)
  }
  fn replay_every(sample: Vec<(Duration, E)>, interval: Duration) -> Self
  where
    E: Clone,
  {
    Self::lazy(move || {
      Self::from_iter(sample.clone()).chain_at(interval, Self::replay_every(sample, interval))
    })
  }
  pub fn take(self, duration: Duration) -> Self {
    self.chain_at(duration, Self::empty())
  }
}

impl<'a, E: 'a> IntoIterator for Stream<'a, E> {
  type Item = (Duration, E);
  type IntoIter = Box<dyn Iterator<Item = (Duration, E)> + 'a>;
  fn into_iter(self) -> Self::IntoIter {
    self.0
  }
}

struct ChainAt<'a, E> {
  source1: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
  source2: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
  threshold: Duration,
}
impl<'a, E: 'a> Iterator for ChainAt<'a, E> {
  type Item = (Duration, E);
  fn next(&mut self) -> Option<(Duration, E)> {
    match self.source1.next() {
      Some((d, e)) if d <= self.threshold => {
        self.threshold -= d;
        Some((d, e))
      }
      _ => {
        self.source1 = Box::new(std::iter::empty());
        match self.source2.next() {
          Some((d, e)) => {
            let d = d + self.threshold;
            self.threshold = Duration::from_secs(0);
            Some((d, e))
          }
          None => None,
        }
      }
    }
  }
}

struct Coalesce<'a, E, F> {
  source: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
  head: Option<(Duration, E)>,
  reduce: F,
}
impl<'a, E, F> Iterator for Coalesce<'a, E, F>
where
  F: Fn(E, E) -> E,
{
  type Item = (Duration, E);
  fn next(&mut self) -> Option<(Duration, E)> {
    match (self.head.take(), self.source.next()) {
      (Some((d1, e1)), Some((d2, e2))) if d2 == Duration::from_secs(0) => {
        self.head = Some((d1, (self.reduce)(e1, e2)));
        self.next()
      }
      (head, next) => {
        self.head = next;
        head
      }
    }
  }
}

struct Drop<'a, E> {
  source: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
  duration: Duration,
}
impl<'a, E> Iterator for Drop<'a, E> {
  type Item = (Duration, E);
  fn next(&mut self) -> Option<(Duration, E)> {
    loop {
      match self.source.next() {
        Some((d, e)) if d >= self.duration => {
          let d = d - self.duration;
          self.duration = Duration::from_secs(0);
          return Some((d, e));
        }
        Some((d, _)) => {
          self.duration -= d;
        }
        None => return None,
      }
    }
  }
}

enum Lazy<'a, E, F> {
  Before(Option<F>),
  After(Box<dyn Iterator<Item = (Duration, E)> + 'a>),
}
impl<'a, E: 'a, F> Iterator for Lazy<'a, E, F>
where
  F: FnOnce() -> Stream<'a, E> + 'a,
{
  type Item = (Duration, E);
  fn next(&mut self) -> Option<(Duration, E)> {
    loop {
      match self {
        Lazy::Before(opt) => {
          *self = Lazy::After(opt.take().unwrap()().into_iter());
        }
        Lazy::After(iter) => return iter.next(),
      }
    }
  }
}

struct Merge<'a, E> {
  head1: Option<(Duration, E)>,
  head2: Option<(Duration, E)>,
  source1: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
  source2: Box<dyn Iterator<Item = (Duration, E)> + 'a>,
}
impl<'a, E: 'a> Iterator for Merge<'a, E> {
  type Item = (Duration, E);
  fn next(&mut self) -> Option<(Duration, E)> {
    match (self.head1.as_mut(), self.head2.as_mut()) {
      (_, None) => std::mem::replace(&mut self.head1, self.source1.next()),
      (None, _) => std::mem::replace(&mut self.head2, self.source2.next()),
      (Some((d1, _)), Some((d2, _))) => {
        if *d1 <= *d2 {
          *d2 -= *d1;
          std::mem::replace(&mut self.head1, self.source1.next())
        } else {
          *d1 -= *d2;
          std::mem::replace(&mut self.head2, self.source2.next())
        }
      }
    }
  }
}
