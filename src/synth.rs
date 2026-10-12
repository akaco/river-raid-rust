//! A tiny two-voice synthesizer: each voice is a square wave or noise, with
//! a pitch and a volume that the game sets every frame.

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Wave {
    #[default]
    Square,
    Noise,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Voice {
    pub wave: Wave,
    /// Hz.
    pub pitch: f32,
    /// 0 to 1.
    pub volume: f32,
}

#[derive(Clone)]
pub struct Synth {
    rate: f32,
    voices: [Voice; 2],
    phase: [f32; 2],
    noise: [u32; 2],
    level: [f32; 2],
}

impl Synth {
    pub fn new(sample_rate: u32) -> Synth {
        Synth {
            rate: sample_rate as f32,
            voices: [Voice::default(); 2],
            phase: [0.0; 2],
            noise: [0x1234_5678, 0x0BAD_F00D],
            level: [0.0; 2],
        }
    }

    pub fn set(&mut self, voices: &[Voice; 2]) {
        self.voices = *voices;
    }

    /// The next sample, -1 to 1.
    pub fn sample(&mut self) -> f32 {
        let mut out = 0.0;
        for i in 0..2 {
            let v = self.voices[i];
            self.phase[i] += v.pitch / self.rate;
            let wrapped = self.phase[i] >= 1.0;
            self.phase[i] %= 1.0;
            let raw = match v.wave {
                Wave::Square => {
                    if self.phase[i] < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                Wave::Noise => {
                    if wrapped {
                        let mut x = self.noise[i];
                        x ^= x << 13;
                        x ^= x >> 17;
                        x ^= x << 5;
                        self.noise[i] = x;
                    }
                    if self.noise[i] & 1 == 0 { 1.0 } else { -1.0 }
                }
            };
            // glide towards the target volume to avoid clicks
            self.level[i] += (v.volume - self.level[i]) * 0.002;
            out += raw * self.level[i];
        }
        out * 0.5
    }
}
