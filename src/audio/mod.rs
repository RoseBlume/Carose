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

pub struct Audio {}

impl Audio {
    pub fn new() -> Self {
        Self { }
    }
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
    /// Create new BGS with initial source
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

    /// Pause/unpause playback
    pub fn playing(&self, play: bool) {
        let sink = self.sink.lock().unwrap();
        if play {
            sink.play();
        } else {
            sink.pause();
        }
    }

    /// Change the looping source dynamically
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