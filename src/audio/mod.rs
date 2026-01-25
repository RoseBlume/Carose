use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source, Sink};
use rodio::source::SineWave;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub enum SoundSource {
    File(&'static str),
    BuiltIn(BuiltInSound),
}

#[derive(Clone)]
pub enum BuiltInSound {
    Shoot,
    Kill,
}

/// Lightweight audio playback utility for one-shot sound effects.
///
/// `Audio` is designed for fire-and-forget sound playback. Each sound
/// is played asynchronously on its own thread and does not require
/// manual lifecycle management.
pub struct Audio {}

impl Audio {
    /// Creates a new audio playback helper.
    ///
    /// This does not allocate or open any audio resources until a sound
    /// is played.
    pub fn new() -> Self {
        Self {}
    }

    /// Plays a sound asynchronously.
    ///
    /// The sound is played on a detached thread and will run to completion
    /// without blocking the caller. This is intended for short sound effects
    /// such as UI feedback, shots, or impacts.
    ///
    /// # Parameters
    /// - `sound`: The sound source to play (file-based or built-in).
    ///
    /// # Notes
    /// - Each call spawns a new thread.
    /// - Audio playback uses a temporary audio stream and sink.
    /// - Playback ends automatically when the sound finishes.
    /// - This method is fire-and-forget; there is no pause or stop control.
    ///
    /// # Panics
    /// Panics if the audio stream or sound source cannot be created.
    pub fn play(&self, sound: SoundSource) {
        thread::spawn(move || {
            // Open audio stream
            let mut stream_handle = OutputStreamBuilder::open_default_stream()
                .expect("Failed to open audio stream");
            // #[cfg(not(debug_assertions))]
            stream_handle.log_on_drop(false);
            // Create sink for playback
            let sink = Sink::connect_new(&stream_handle.mixer());

            // Determine source and its duration
            let _duration: Duration = match sound {
                SoundSource::File(path) => {
                    let file = BufReader::new(File::open(path).expect("Failed to open file"));
                    let source = Decoder::new(file).expect("Failed to decode file");
                    let dur = source.total_duration().unwrap_or(Duration::from_secs_f32(0.25));
                    sink.append(source);
                    dur
                }
                SoundSource::BuiltIn(builtin) => {
                    let source = match builtin {
                        BuiltInSound::Shoot => SineWave::new(880.0)
                            .take_duration(Duration::from_secs_f32(0.06))
                            .amplify(0.25),
                        BuiltInSound::Kill => SineWave::new(220.0)
                            .take_duration(Duration::from_secs_f32(0.12))
                            .amplify(0.25),
                    };
                    let dur = source.total_duration().unwrap_or(Duration::from_secs_f32(0.25));
                    sink.append(source);
                    dur
                }
            };

            // Sleep the thread exactly for the duration of the sound
            sink.sleep_until_end();
        });
    }
}




/// Background sound player that loops until paused or dropped
pub struct Bgs {
    _stream: OutputStream,                 // keep alive
    stream_handle: OutputStream,           // handle for sinks
    sink: Arc<Mutex<Sink>>,                // current sink
    source: Arc<Mutex<SoundSource>>,       // current source

    playlist: Arc<Mutex<Vec<SoundSource>>>,
    playlist_index: Arc<Mutex<usize>>,
}


impl Bgs {
    /// Creates a new background sound system with a single looping source.
    ///
    /// This initializes an audio output stream, creates an internal sink,
    /// and immediately begins playback of the provided sound source.
    ///
    /// The initial source is also inserted into a playlist, allowing the
    /// background sound system to be extended later into a rotating playlist.
    ///
    /// # Parameters
    /// - `initial_source`: The sound source to loop and play immediately.
    ///
    /// # Panics
    /// Panics if the default audio output stream or sink cannot be created.
    ///
    /// # Notes
    /// - Playback starts automatically.
    /// - The source is looped infinitely.
    pub fn new(initial_source: SoundSource) -> Self {
        let _stream = OutputStreamBuilder::open_default_stream().expect("Failed stream");
        let stream_handle = OutputStreamBuilder::open_default_stream().expect("Failed handle");
        let sink = Sink::connect_new(&stream_handle.mixer());
        let sink_arc = Arc::new(Mutex::new(sink));

        let bgs = Self {
            _stream,
            stream_handle,
            sink: sink_arc.clone(),
            source: Arc::new(Mutex::new(initial_source.clone())),
            playlist: Arc::new(Mutex::new(vec![initial_source.clone()])),
            playlist_index: Arc::new(Mutex::new(0)),
        };

        bgs.playing(true); // start immediately
        bgs.set_source(initial_source); // append looping source

        bgs
    }

    /// Creates a new background sound system using a playlist of sound sources.
    ///
    /// The playlist will automatically advance when the current source finishes.
    /// Playback begins immediately, starting from the first source in the list.
    ///
    /// # Parameters
    /// - `sources`: A non-empty list of sound sources to play in sequence.
    ///
    /// # Panics
    /// Panics if `sources` is empty or if the audio stream cannot be created.
    ///
    /// # Notes
    /// - The playlist loops infinitely.
    /// - Sources are played in the order provided.
    pub fn playlist(sources: Vec<SoundSource>) -> Self {
        assert!(!sources.is_empty(), "Playlist cannot be empty");

        let _stream = OutputStreamBuilder::open_default_stream().expect("Failed stream");
        let stream_handle = OutputStreamBuilder::open_default_stream().expect("Failed handle");

        let sink = Sink::connect_new(&stream_handle.mixer());
        let sink_arc = Arc::new(Mutex::new(sink));
        let first = sources[0].clone();

        let bgs = Self {
            _stream,
            stream_handle,
            sink: sink_arc.clone(),
            source: Arc::new(Mutex::new(first.clone())),
            playlist: Arc::new(Mutex::new(sources)),
            playlist_index: Arc::new(Mutex::new(0)),
        };

        bgs.playing(true);
        bgs.set_source(first);

        bgs
    }
    /// Advances the playlist if the current sound has finished playing.
    ///
    /// This should be called regularly (e.g., once per frame or tick).
    /// If the sink is empty, the playlist index is advanced and the next
    /// source is loaded and played.
    ///
    /// # Behavior
    /// - If the current source is still playing, this method does nothing.
    /// - If the end of the playlist is reached, playback wraps to the beginning.
    pub fn update_playlist(&self) {
        let should_advance = {
            let sink = self.sink.lock().unwrap();
            sink.empty()
        };

        if !should_advance {
            return;
        }

        let mut index = self.playlist_index.lock().unwrap();
        let playlist = self.playlist.lock().unwrap();

        *index = (*index + 1) % playlist.len();
        let next = playlist[*index].clone();

        drop(playlist);
        drop(index);

        self.set_source(next);
    }

    /// Pauses or resumes playback.
    ///
    /// # Parameters
    /// - `play`: If `true`, playback resumes or continues.
    ///           If `false`, playback is paused.
    ///
    /// # Notes
    /// - This does not reset the current sound.
    /// - The playback position is preserved when pausing.
    pub fn playing(&self, play: bool) {
        let sink = self.sink.lock().unwrap();
        if play {
            sink.play();
        } else {
            sink.pause();
        }
    }

    /// Replaces the currently playing source with a new looping source.
    ///
    /// The existing sink is stopped and replaced with a fresh sink
    /// connected to the same output stream. The new source begins
    /// playing immediately and loops infinitely.
    ///
    /// # Parameters
    /// - `new_source`: The sound source to load and play.
    ///
    /// # Notes
    /// - This method is thread-safe.
    /// - File-based sources are decoded at load time.
    /// - Built-in sounds are procedurally generated.
    ///
    /// # Panics
    /// Panics if a sound file cannot be opened or decoded.
    pub fn set_source(&self, new_source: SoundSource) {
        let mut source_lock = self.source.lock().unwrap();
        *source_lock = new_source.clone();

        // Stop current sink and create a new one
        let mut sink_lock = self.sink.lock().unwrap();
        sink_lock.stop();

        let new_sink = Sink::connect_new(&self.stream_handle.mixer());

        let rodio_source: Box<dyn Source<Item = f32> + Send> = match new_source {
            SoundSource::File(path) => {
                let file = File::open(path).expect("Failed to open file");
                let decoder = Decoder::new(BufReader::new(file)).expect("Failed to decode file");
                Box::new(decoder.repeat_infinite())
            }
            SoundSource::BuiltIn(builtin) => {
                match builtin {
                    BuiltInSound::Shoot => Box::new(
                        rodio::source::SineWave::new(880.0)
                            .take_duration(Duration::from_secs_f32(0.06))
                            .repeat_infinite()
                            .amplify(0.25),
                    ),
                    BuiltInSound::Kill => Box::new(
                        rodio::source::SineWave::new(220.0)
                            .take_duration(Duration::from_secs_f32(0.12))
                            .repeat_infinite()
                            .amplify(0.25),
                    ),
                }
            }
        };

        new_sink.append(rodio_source);
        new_sink.play();
        *sink_lock = new_sink;
    }
}