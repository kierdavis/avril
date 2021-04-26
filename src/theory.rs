#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PitchClass {
  C,
  CSharp,
  D,
  DSharp,
  E,
  F,
  FSharp,
  G,
  GSharp,
  A,
  ASharp,
  B,
}

impl PitchClass {
  fn ordinal(self) -> i64 {
    match self {
      Self::C => 0,
      Self::CSharp => 1,
      Self::D => 2,
      Self::DSharp => 3,
      Self::E => 4,
      Self::F => 5,
      Self::FSharp => 6,
      Self::G => 7,
      Self::GSharp => 8,
      Self::A => 9,
      Self::ASharp => 10,
      Self::B => 11,
    }
  }
  fn from_ordinal(ordinal: i64) -> Self {
    match ordinal.rem_euclid(12) {
      0 => Self::C,
      1 => Self::CSharp,
      2 => Self::D,
      3 => Self::DSharp,
      4 => Self::E,
      5 => Self::F,
      6 => Self::FSharp,
      7 => Self::G,
      8 => Self::GSharp,
      9 => Self::A,
      10 => Self::ASharp,
      11 => Self::B,
      _ => unreachable!(),
    }
  }
}

impl std::fmt::Display for PitchClass {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str(match self {
      Self::C => "C",
      Self::CSharp => "C#",
      Self::D => "D",
      Self::DSharp => "D#",
      Self::E => "E",
      Self::F => "F",
      Self::FSharp => "F#",
      Self::G => "G",
      Self::GSharp => "G#",
      Self::A => "A",
      Self::ASharp => "A#",
      Self::B => "B",
    })
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Note {
  // Relative to middle C (C above 440 Hz).
  semitones: i64,
}
impl Note {
  pub fn new(pitch_class: PitchClass, octave: i64) -> Self {
    Note {
      semitones: pitch_class.ordinal() + (octave - 4) * 12,
    }
  }
  pub fn pitch_class(self) -> PitchClass {
    PitchClass::from_ordinal(self.semitones)
  }
  pub fn octave(self) -> i64 {
    self.semitones.div_euclid(12) + 4
  }
  pub fn offset(self, semitones: i64) -> Self {
    Note {
      semitones: self.semitones + semitones,
    }
  }
  pub fn midi(self) -> u8 {
    let value = self.semitones + 60;
    if value < 0 || value > 127 {
      panic!("midi note out of range");
    }
    value as u8
  }
}

impl std::fmt::Display for Note {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}{}", self.pitch_class(), self.octave())
  }
}

#[test]
fn test_note() {
  use PitchClass::*;
  assert_eq!(Note::new(C, 4).semitones, 0);
  assert_eq!(Note::new(C, 5).semitones, 12);
  assert_eq!(Note::new(C, 3).semitones, -12);
  assert_eq!(Note::new(D, 4).semitones, 2);
  assert_eq!(Note::new(D, 3).semitones, -10);
  assert_eq!(Note::new(B, 4).semitones, 11);
  assert_eq!(Note::new(B, 3).semitones, -1);

  assert_eq!(Note::new(D, 4).offset(0), Note::new(D, 4));
  assert_eq!(Note::new(D, 4).offset(1), Note::new(DSharp, 4));
  assert_eq!(Note::new(D, 4).offset(10), Note::new(C, 5));
  assert_eq!(Note::new(D, 4).offset(12), Note::new(D, 5));
  assert_eq!(Note::new(D, 4).offset(-1), Note::new(CSharp, 4));
  assert_eq!(Note::new(D, 4).offset(-3), Note::new(B, 3));
  assert_eq!(Note::new(D, 4).offset(-12), Note::new(D, 3));
}

// Semitone intervals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scale(Vec<i64>);

impl Scale {
  pub fn from_intervals(intervals: Vec<i64>) -> Self {
    assert_eq!(intervals.iter().sum::<i64>(), 12);
    Self(intervals)
  }
  pub fn major() -> Self {
    Self::from_intervals(vec![2, 2, 1, 2, 2, 2, 1])
  }
  pub fn minor() -> Self {
    Self::from_intervals(vec![2, 1, 2, 2, 1, 2, 2])
  }
  pub fn pentatonic() -> Self {
    Self::from_intervals(vec![4, 1, 2, 4, 1])
  }
  pub fn num_intervals(&self) -> usize {
    self.0.len()
  }
  pub fn intervals_ascending<'a>(&'a self) -> impl Iterator<Item = i64> + 'a {
    self.0.iter().copied().cycle()
  }
  pub fn intervals_descending<'a>(&'a self) -> impl Iterator<Item = i64> + 'a {
    self.0.iter().rev().map(|x| -x).cycle()
  }
}

#[test]
fn test_scale() {
  assert_eq!(
    Scale::major()
      .intervals_ascending()
      .take(10)
      .collect::<Vec<_>>(),
    vec![2, 2, 1, 2, 2, 2, 1, 2, 2, 1]
  );
  assert_eq!(
    Scale::major()
      .intervals_descending()
      .take(10)
      .collect::<Vec<_>>(),
    vec![-1, -2, -2, -2, -1, -2, -2, -1, -2, -2]
  );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Key {
  tonic: Note,
  scale: Scale,
}

impl Key {
  pub fn major(tonic: Note) -> Self {
    Self {
      tonic,
      scale: Scale::major(),
    }
  }
  pub fn minor(tonic: Note) -> Self {
    Self {
      tonic,
      scale: Scale::minor(),
    }
  }
  pub fn pentatonic(tonic: Note) -> Self {
    Self {
      tonic,
      scale: Scale::pentatonic(),
    }
  }
  pub fn offset_tonic(self, scale_steps: i64) -> Self {
    Self {
      tonic: self.at(scale_steps).note(),
      scale: self.scale,
    }
  }
  pub fn scale(&self) -> &Scale {
    &self.scale
  }
  pub fn notes_ascending<'a>(&'a self) -> impl Iterator<Item = NoteInKey<'a>> + 'a {
    self.notes_from_intervals(self.scale.intervals_ascending(), 1)
  }
  pub fn notes_descending<'a>(&'a self) -> impl Iterator<Item = NoteInKey<'a>> + 'a {
    self.notes_from_intervals(self.scale.intervals_descending(), -1)
  }
  fn notes_from_intervals<'a, I: Iterator<Item = i64> + 'a>(
    &'a self,
    intervals: I,
    direction: i64,
  ) -> impl Iterator<Item = NoteInKey<'a>> + 'a {
    intervals
      .enumerate()
      .scan(self.tonic, move |accum, (i, interval)| {
        let note = *accum;
        *accum = note.offset(interval);
        Some(NoteInKey {
          key: self,
          note,
          scale_steps: (i as i64) * direction,
        })
      })
  }
  pub fn at<'a>(&'a self, scale_steps_from_tonic: i64) -> NoteInKey<'a> {
    if scale_steps_from_tonic >= 0 {
      self
        .notes_ascending()
        .nth(scale_steps_from_tonic as usize)
        .unwrap()
    } else {
      self
        .notes_descending()
        .nth(-scale_steps_from_tonic as usize)
        .unwrap()
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoteInKey<'k> {
  key: &'k Key,
  note: Note,
  scale_steps: i64, // relative to tonic of key
}

impl<'k> NoteInKey<'k> {
  pub fn offset(&self, scale_steps: i64) -> NoteInKey<'k> {
    self.key.at(self.scale_steps + scale_steps)
  }
  pub fn note(&self) -> Note {
    self.note
  }
  pub fn scale_steps_from_tonic(&self) -> i64 {
    self.scale_steps
  }
}

impl<'k> std::fmt::Display for NoteInKey<'k> {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    std::fmt::Display::fmt(&self.note, f)
  }
}

#[test]
fn test_key() {
  use PitchClass::*;
  let key = Key::minor(Note::new(GSharp, 3));
  assert_eq!(
    key
      .notes_ascending()
      .take(10)
      .map(|nk| nk.note)
      .collect::<Vec<_>>(),
    vec![
      Note::new(GSharp, 3),
      Note::new(ASharp, 3),
      Note::new(B, 3),
      Note::new(CSharp, 4),
      Note::new(DSharp, 4),
      Note::new(E, 4),
      Note::new(FSharp, 4),
      Note::new(GSharp, 4),
      Note::new(ASharp, 4),
      Note::new(B, 4),
    ]
  );
  assert_eq!(
    key
      .notes_descending()
      .take(10)
      .map(|nk| nk.note)
      .collect::<Vec<_>>(),
    vec![
      Note::new(GSharp, 3),
      Note::new(FSharp, 3),
      Note::new(E, 3),
      Note::new(DSharp, 3),
      Note::new(CSharp, 3),
      Note::new(B, 2),
      Note::new(ASharp, 2),
      Note::new(GSharp, 2),
      Note::new(FSharp, 2),
      Note::new(E, 2),
    ]
  );
}
