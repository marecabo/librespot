use base64;
use std::f32;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use super::{Mixer, MixerConfig};

#[derive(Clone)]
pub struct PipeMixer {
    volume: Arc<AtomicUsize>,
    pipe: Option<String>,
}

impl PipeMixer {
    fn apply_volume(&self, vol: u16) {
        if let Some(path) = self.pipe.clone() {
            let metadata_vol = if vol == 0 {
                -144.0f32
            } else if vol == 1 {
                -30.0f32
            } else if vol == 0xFFFF {
                0.0f32
            } else {
                ((vol as f32) - (0xFFFF as f32)) * 30.0f32 / (0xFFFE as f32)
            };

            let vol_string = format!("{:.*},0.00,0.00,0.00", 2, metadata_vol);
            let vol_string_len = vol_string.chars().count();
            let metadata_vol_string = base64::encode(&vol_string);
            let metadata_xml = format!("<item><type>73736e63</type><code>70766f6c</code><length>{}</length>\n<data encoding=\"base64\">\n{}</data></item>", vol_string_len, metadata_vol_string);

            thread::spawn(move || {
                let mut f = File::create(&path).expect("Unable to open pipe");
                f.write_all(metadata_xml.as_bytes()).expect("Unable to write data");
            });
        }
    }
}

impl Mixer for PipeMixer {
    fn open(config: Option<MixerConfig>) -> PipeMixer {
        PipeMixer {
            volume: Arc::new(AtomicUsize::new(0xFFFF)),
            pipe: config.unwrap_or_default().pipe,
        }
    }
    fn start(&self) {}

    fn stop(&self) {}

    fn volume(&self) -> u16 {
        self.volume.load(Ordering::Relaxed) as u16
    }

    fn set_volume(&self, volume: u16) {
        self.volume.store(volume as usize, Ordering::Relaxed);
        self.apply_volume(volume);
    }
}
