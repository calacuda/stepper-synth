from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


def draw_lp_menu(pygame, screen, fonts, synth: StepperSynthState):
    pass


def lp_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    # move_cursor(controller)
    # return adjust_value(pygame, controller, synth, state)
    return synth
