# Stepper Synth

A multi-mode synth for retro emulation consoles running knulli/batocera. It supports multiple [synth engines](#synth-engines) and [effects](#effects) to get different sounds. (Any effect can be applied to any synth-engine.) It also has a global LFO to modulate any of the many parameters of the synth engine or the effect.

## Synth Engines

- Organ synth (like a digital Hammond B3)
- Subtractive synth (standard subtractive synth with a Moog style ladder low-pass filter)
- Sampler Synth (plays samples)
- Modelling Synth

## Effects

- Reverb
- Delay

## Progress

- [x] Oragn Synth engine
- [x] Subtractive Synth engine
- [x] Organ Synth ui
- [x] Subtractive Synth ui
- [ ] Sampler Synth
- [ ] Sampler Synth ui
- [x] synth switch menu
- [x] effect switch menu
- [ ] volume & battery icons
- [x] reverb ui
- [ ] chorus ui
- [x] better delay
- [ ] delay ui
- [ ] LFO ui
- [x] LFO Routing
- [x] low_pass keytracking
- [ ] Drum synth
- [x] Wave Table Synth
- [x] Wave Table Synth ui
- [ ] config file
- [ ] maybe a CLI (emphasis on maybe)
- [ ] midi sequencer
- [x] mod matrix
- [ ] wurlitzer engine
- [ ] wurlitzer UI
- [ ] overdrive effect.
- [ ] add midi bindings
- [x] finish screens for wavetable synth
  - [ ] mod matrix edit screen is buggy
- [ ] update midi stepper to have 4 "channels" each with the same number of steps, they should play in unison
  - [ ] rming notes from non main channel is broken
- [ ] each midi stepper channel should be routed to a different instrument.
- [ ] loading premade wave tables.
- [ ] making wave tables.
- [ ] saving wave tables made in the software for later.

## Currently Working On

- enable editing step number and tempo in stepper ui 

## TODOs

- [x] improve wave table synth efficiency.
  - [x] global LFOs 
  - [x] global effects 
  - [x] only most recent note in the wave table.
- [ ] generate audio buffer before its needed.
- [ ] update midi stepper to have 4 "channels" each with the same number of steps, they should play in unison
- [ ] each midi stepper channel should be routed to a different instrument (configurable).
- [ ] add a way to for the wavetable synth to learn midi bindings.
