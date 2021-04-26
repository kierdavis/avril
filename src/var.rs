use crate::stream::Stream;
use std::time::Duration;

pub struct Var<'a, T> {
  present: T,
  future: Stream<'a, T>,
}

impl<'a, T> Var<'a, T> {
  pub fn constant(value: T) -> Self {
    Self {
      present: value,
      future: Stream::empty(),
    }
  }
  pub fn from_updates(initial_value: T, updates: Stream<'a, T>) -> Self {
    Self {
      present: initial_value,
      future: updates,
    }
  }
  pub fn updates(self) -> Stream<'a, T> {
    Stream::immediate(self.present).chain(self.future)
  }
  pub fn map<F, U>(self, mut func: F) -> Var<'a, U>
  where
    F: FnMut(T) -> U + 'a,
  {
    Var {
      present: func(self.present),
      future: self.future.map(func),
    }
  }
  pub fn repeat_every(self, interval: Duration) -> Self
  where
    T: Clone,
  {
    Self {
      present: self.present.clone(),
      future: self.updates().repeat_every(interval),
    }
  }
}

impl<'a, T> Var<'a, Stream<'a, T>> {
  pub fn sequence(self) -> Stream<'a, T> {
    Stream::lazy(move || {
      let Var {
        present,
        mut future,
      } = self;
      match future.next() {
        Some((d, s)) => present.chain_at(d, Var { present: s, future }.sequence()),
        None => present,
      }
    })
  }
}

impl<'a, T> Var<'a, Var<'a, T>> {
  pub fn sequence(self) -> Var<'a, T> {
    let mut future = self.map(Var::updates).sequence();
    match future.next() {
      Some((dur, present)) if dur == Duration::from_secs(0) => Var { present, future },
      _ => unreachable!(),
    }
  }
}

impl<'a, T> AsRef<T> for Var<'a, T> {
  fn as_ref(&self) -> &T {
    &self.present
  }
}
