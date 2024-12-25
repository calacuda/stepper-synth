from .controls import Buttons, buttons
from stepper_synth_backend import StepperSynthState, StepperSynth, StepCmd
from .config import *
from .utils import *
from dataclasses import dataclass


@dataclass()
class Key:
    note: int
    white_key: bool


KEYS = [
    Key(0, True),
    Key(1, False),
    Key(2, True),
    Key(3, False),
    Key(4, True),
    Key(5, True),
    Key(6, False),
    Key(7, True),
    Key(8, False),
    Key(9, True),
    Key(10, False),
    Key(11, True),
]


def draw_octave(pygame, screen, state: StepperSynthState, top: float, width: float, offset: float, octave_n: int, playing):
    key_width = width / 7
    half_width = (key_width) * 0.5
    bottom = SCREEN_HEIGHT - LINE_WIDTH
    w_height = SCREEN_HEIGHT - top - LINE_WIDTH
    b_height = (SCREEN_HEIGHT - top) * 0.5 - LINE_WIDTH
    white_keys_found = 0
    black_keys_found = 1
    b_width = width / 12

    # for (i, key) in enumerate([key for key in KEYS if key.white_key]):
    for key in KEYS:
        if key.white_key:
            left = key_width * white_keys_found + offset
            w = key_width

            midi_note = octave_n * 12 + key.note
            color = SURFACE_0 if midi_note in playing else TEXT

            pygame.draw.rect(screen, SURFACE_2, pygame.Rect(
                left, top, w + LINE_WIDTH, w_height))
            rect = pygame.Rect(
                left + LINE_WIDTH, top + LINE_WIDTH, w - LINE_WIDTH, w_height - LINE_WIDTH * 2)
            rect.top = top + LINE_WIDTH
            pygame.draw.rect(screen, color, rect)
            white_keys_found += 1

    # for (i, key) in enumerate(KEYS):
    for (i, key) in enumerate(KEYS):
        if not key.white_key:

            left = b_width * i + offset + LINE_WIDTH / 2

            midi_note = octave_n * 12 + key.note
            color = SURFACE_0 if midi_note in playing else CRUST

            rect = pygame.Rect(
                left, top, b_width + LINE_WIDTH, b_height + LINE_WIDTH)
            # rect.centerx = left
            pygame.draw.rect(screen, SURFACE_2, rect)

            rect = pygame.Rect(
                left + LINE_WIDTH, top + LINE_WIDTH, b_width - LINE_WIDTH, b_height - LINE_WIDTH)
            rect.top = top + LINE_WIDTH
            # rect.centerx = left
            pygame.draw.rect(screen, color, rect)
            black_keys_found += 1


def draw_piano(pygame, screen, state: StepperSynthState, top, playing):
    octave_w = (SCREEN_WIDTH - LINE_WIDTH * 3) / 10

    for i in range(10):
        draw_octave(pygame, screen, state, top + LINE_WIDTH, octave_w,
                    octave_w * i + LINE_WIDTH, i, playing)
        # break


def draw_stepper(pygame, screen, fonts, state: StepperSynthState):
    third = SCREEN_HEIGHT / 3
    bottom_h = third * 2
    bottom_row_h = bottom_h / 3

    playing = state.step.on_enter  # if state.playing else []
    # print("first_playing =", playing)
    playing = [step.note for (_, step) in playing if isinstance(
        step, StepCmd.Play)]
    # print("second_playing =", playing)

    draw_piano(pygame, screen, state, SCREEN_HEIGHT - bottom_row_h, playing)
    # print("test")


def reverb_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    pass
