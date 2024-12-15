use crate::SAMPLE_RATE;

static UNPRESSED: usize = 0;
static ATTACK: usize = 1;
static DECAY: usize = 2;
static SUSTAIN: usize = 3;
static RELEASE: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct ADSR {
    pub sample_rate: u32,
    pub phase: usize,
    base_params: [f32; 5],
    tweek_env_by: [f32; 5],
    env: f32,
}

impl ADSR {
    pub fn new() -> Self {
        let base_params = [0.0, 0.1, 0.1, 0.5, 0.1];

        Self {
            sample_rate: SAMPLE_RATE,
            phase: 0,
            base_params,
            tweek_env_by: Self::calc_tweek_by(base_params),
            env: 0.0,
        }
    }

    fn calc_tweek_by(base_params: [f32; 5]) -> [f32; 5] {
        let mut tweek_env_by = [0.0; 5];

        tweek_env_by[ATTACK] = Self::calc_atk(base_params[ATTACK]);
        tweek_env_by[DECAY] = Self::calc_decay(base_params[DECAY], base_params[SUSTAIN]);
        tweek_env_by[RELEASE] = Self::calc_release(base_params[RELEASE], base_params[SUSTAIN]);

        // println!("{}", tweek_env_by[DECAY]);

        tweek_env_by
    }

    fn calc_atk(atk_speed: f32) -> f32 {
        1.0 / (atk_speed * SAMPLE_RATE as f32)
    }

    fn calc_decay(decay_speed: f32, sustain_level: f32) -> f32 {
        (-1.0 + sustain_level) / (decay_speed * SAMPLE_RATE as f32)
    }

    fn calc_release(release_speed: f32, sustain_level: f32) -> f32 {
        (-1.0 + sustain_level) / (release_speed * SAMPLE_RATE as f32)
    }

    pub fn set_atk(&mut self, atk: f32) {
        // set attack
        self.base_params[ATTACK] = atk;

        self.tweek_env_by[ATTACK] = Self::calc_atk(atk);
    }

    pub fn set_decay(&mut self, decay: f32) {
        // set decay
        self.base_params[DECAY] = decay;

        self.tweek_env_by[DECAY] = Self::calc_decay(decay, self.base_params[SUSTAIN]);
    }

    pub fn set_sus(&mut self, sustain: f32) {
        // set sustain
        self.base_params[SUSTAIN] = sustain;

        self.tweek_env_by[DECAY] =
            Self::calc_decay(self.base_params[DECAY], self.base_params[SUSTAIN]);
        self.tweek_env_by[RELEASE] =
            Self::calc_release(self.base_params[RELEASE], self.base_params[SUSTAIN]);
    }

    pub fn set_release(&mut self, release: f32) {
        let release = release * 0.25;

        self.base_params[RELEASE] = release;
        self.tweek_env_by[RELEASE] = Self::calc_release(release, self.base_params[SUSTAIN]);
    }

    /// used to generate an env sample
    pub fn get_samnple(&mut self) -> f32 {
        self.env += self.tweek_env_by[self.phase];
        // println!("tweak => {}", self.tweek_env_by[self.phase]);
        // println!("phase => {}", self.phase);

        if self.env > 1.0 && self.phase == ATTACK {
            // println!("now decay");
            self.phase = DECAY;
            self.env = 1.0;
            // println!("tweak => {}", self.tweek_env_by[self.phase]);
            // println!("phase => {}", self.phase);
            // println!("env => {}", self.env);
        } else if self.env < self.base_params[SUSTAIN] && self.phase == DECAY {
            // println!("now sustain");
            self.phase = SUSTAIN;
            self.env = self.base_params[SUSTAIN];
            // println!("tweak => {}", self.tweek_env_by[self.phase]);
            // println!("phase => {}", self.phase);
            // println!("env => {}", self.env);
        } else if self.env <= 0.0 {
            // println!("now released");
            self.phase = RELEASE;
            self.env = 0.0;
        }

        // if self.phase == RELEASE {
        //     println!("RELEASE");
        //     println!("tweak => {}", self.tweek_env_by[self.phase]);
        //     println!("phase => {}", self.phase);
        //     println!("env => {}", self.env);
        // }

        self.env
    }

    /// presses the key
    pub fn press(&mut self) {
        self.phase = ATTACK;
        self.env = 0.0;
    }

    /// Release the key if pressed
    pub fn release(&mut self) {
        self.phase = RELEASE;
        self.env = self.base_params[SUSTAIN];
    }

    /// returns true if the env filter is not released
    pub fn pressed(&self) -> bool {
        self.phase != RELEASE && self.phase != UNPRESSED
    }
}
