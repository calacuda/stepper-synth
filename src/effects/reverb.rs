use reverb;

pub struct Reverb {
    pub effect: reverb::Reverb,
    pub gain: f32,
    pub decay: f32,
    pub power: bool,
}

impl Reverb {
    pub fn new() -> Self {
        Self {
            effect: reverb::Reverb::new(),
            gain: 0.5,
            decay: 0.5,
            power: false,
        }
    }

    pub fn get_sample(&mut self, in_sample: f32) -> f32 {
        if self.power {
            self.effect.calc_sample(in_sample, self.gain)
        } else {
            in_sample
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay;

        self.effect = self.effect.decay(decay).clone();
    }

    pub fn turn_power_on(&mut self, power: bool) {
        self.power = power;
    }

    pub fn power_toggle(&mut self) {
        self.power = !self.power;
    }
}
