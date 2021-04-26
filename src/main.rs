use self::midi::MessageExt;
use self::seed::Seed;
use self::stream::Stream;
use self::theory::{Key, Note, NoteInKey, PitchClass};
use self::var::Var;
use rand_distr::{Distribution, Exp, Normal};
use std::time::Duration;

mod midi;
mod seed;
mod stream;
mod theory;
mod var;

fn melody<'k>(
  key: &'k Key,
  first_note: NoteInKey<'k>,
  quantum_duration: Duration,
  seed: Seed,
) -> Var<'k, NoteInKey<'k>> {
  let delta_std_dev = (key.scale().num_intervals() as f64) / 2.0;
  let num_quanta_distr = Exp::<f64>::new(2.0).unwrap();
  Var::from_updates(
    first_note,
    Stream::from_iter(itertools::unfold(
      (first_note, seed.fork("notes")),
      move |(prev_note, seed)| {
        let delta_distr = Normal::<f64>::new(
          (-prev_note.scale_steps_from_tonic() / 2) as f64,
          delta_std_dev,
        )
        .unwrap();
        let delta = delta_distr
          .sample_iter(&mut seed.fork("delta").rng())
          .map(|x| x.round() as i64)
          .filter(|&x| x != 0)
          .next()
          .unwrap();
        let note = prev_note.offset(delta);
        let num_quanta = num_quanta_distr
          .sample(&mut seed.fork("num_quanta").rng())
          .ceil() as u32;
        let duration = quantum_duration * num_quanta;
        *prev_note = note;
        *seed = seed.fork("next");
        Some((duration, note))
      },
    )),
  )
}

fn play<'a>(channel: midi::Channel, pitch: Var<'a, Option<Note>>) -> Stream<'a, midi::Message> {
  use std::mem::replace;
  let mut current_pitch = None;
  Stream::immediate(midi::Message::AllSoundOff(channel)).chain(
    pitch
      .updates()
      .chain(Stream::immediate(None))
      .coalesce(|_, new| new)
      .flat_map(move |new_pitch| {
        swap_pitch(replace(&mut current_pitch, new_pitch), new_pitch, channel)
      }),
  )
}
fn swap_pitch(old: Option<Note>, new: Option<Note>, channel: midi::Channel) -> Vec<midi::Message> {
  const VELOCITY: u8 = 0x40;
  let mut msgs = Vec::new();
  if let Some(note) = old {
    msgs.push(midi::Message::NoteOff(channel, note.midi(), VELOCITY));
  }
  if let Some(note) = new {
    msgs.push(midi::Message::NoteOn(channel, note.midi(), VELOCITY));
  }
  msgs
}

fn active_sensing() -> Stream<'static, midi::Message> {
  Stream::immediate(midi::Message::ActiveSensing).repeat_every(Duration::from_millis(250))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let seed = Seed::new("frosted glass");
  let beat_duration = Duration::from_millis(230);
  let phrase_duration = beat_duration * 16;
  let phrase_repetitions = 2;
  let reseed_interval = phrase_duration * phrase_repetitions;
  let num_phrases = 6;
  let key = Key::pentatonic(Note::new(PitchClass::D, 4));

  let treble_seed = seed.fork("treble");
  let treble = Var::from_updates(
    treble_seed.fork(0),
    Stream::from_iter((1..).map(|i| (reseed_interval, treble_seed.fork(i)))),
  )
  .map(|mel_seed| melody(&key, key.at(7), beat_duration, mel_seed).repeat_every(phrase_duration))
  .sequence()
  .map(|note_in_key| Some(note_in_key.note()));

  let bass_seed = seed.fork("bass");
  let bass = Var::from_updates(
    bass_seed.fork(0),
    Stream::from_iter((1..).map(|i| (reseed_interval, bass_seed.fork(i)))),
  )
  .map(|mel_seed| {
    melody(&key, key.at(-10), beat_duration * 2, mel_seed).repeat_every(phrase_duration)
  })
  .sequence()
  .map(|note_in_key| Some(note_in_key.note()));

  let messages = Stream::merge_all(vec![
    Stream::immediate(midi::Message::ProgramChange(midi::Channel::Ch1, 0)),
    Stream::immediate(midi::Message::ProgramChange(midi::Channel::Ch2, 0)),
    play(midi::Channel::Ch1, treble),
    play(midi::Channel::Ch2, bass),
    active_sensing(),
  ]);
  let messages = messages.take(phrase_duration * num_phrases);

  let output = midir::MidiOutput::new("avril")?;
  let ports = output.ports();
  let port = ports
    .iter()
    .filter(|port| {
      output
        .port_name(port)
        .unwrap_or(String::new())
        .starts_with("FLUID")
    })
    .next()
    .unwrap();
  let mut conn = output.connect(port, "avril_port")?;
  for (delay, message) in messages {
    println!("{} {:?}", delay.as_millis(), message);
    std::thread::sleep(delay);
    conn.send(&message.encode())?;
  }

  Ok(())
}
