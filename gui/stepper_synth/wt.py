from .wt_osc_menu import draw_osc_menu, osc_menu_controls
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial
from stepper_synth_backend import StepperSynthState, StepperSynth


SUB_SCREENS = [
    (draw_osc_menu, osc_menu_controls),
]
SUB_SCREEN = 0


def draw_wave_table(pygame, screen, fonts, synth: StepperSynthState):
    screen_draw_f = SUB_SCREENS[SUB_SCREEN][0]
    screen_draw_f(pygame, screen, fonts, synth)


def wave_table_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    global SUB_SCREEN

    # if select_mod_pressed(controller):
    #     return

    # right_pressed = controller.just_pressed(buttons.get("right"))
    # left_pressed = controller.just_pressed(buttons.get("left"))
    # a_pressed = controller.just_pressed(buttons.get("a"))
    #
    # if select_mod_pressed(controller) and right_pressed and a_pressed:
    #     SUB_SCREEN += 1
    #     SUB_SCREEN %= len(SUB_SCREENS)
    # elif select_mod_pressed(controller) and left_pressed and a_pressed:
    #     SUB_SCREEN -= 1
    #     SUB_SCREEN %= len(SUB_SCREENS)
    # else:
    synth = SUB_SCREENS[SUB_SCREEN][1](pygame, controller, synth, state)

    return synth
