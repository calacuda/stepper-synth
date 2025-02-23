from .wt_menu import draw_wt_menu, wt_menu_controls
from .wt_osc_menu import draw_osc_menu, osc_menu_controls
from .wt_lp_menu import draw_lp_menu, lp_menu_controls
from .wt_env_menu import draw_env_menu, env_menu_controls
from .wt_mod_menu import draw_mod_menu, mod_menu_controls
from .controls import Buttons
from .config import *
from .utils import *
from stepper_synth_backend import StepperSynthState, StepperSynth


SUB_SCREENS = [
    (draw_osc_menu, osc_menu_controls, "Osc."),
    (draw_lp_menu, lp_menu_controls, "LowPass"),
    (draw_env_menu, env_menu_controls, "Env."),
    (None, None, "LFO"),
    (draw_mod_menu, mod_menu_controls, "Mod"),
]
SCREEN_NAMES = [screen[2] for screen in SUB_SCREENS]
# SUB_SCREEN = 0
SUB_SCREEN = 2
DRAW_MENU = False
# DRAW_MENU = True


def draw_wave_table(pygame, screen, fonts, synth: StepperSynthState):
    screen_draw_f = SUB_SCREENS[SUB_SCREEN][0]

    if screen_draw_f is not None:
        screen_draw_f(pygame, screen, fonts, synth)

    if DRAW_MENU:
        draw_wt_menu(pygame, screen, fonts, SCREEN_NAMES)


def wave_table_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    global SUB_SCREEN
    global DRAW_MENU

    if controller.just_pressed(buttons.get("y")):
        DRAW_MENU = not DRAW_MENU
    elif controller.just_pressed(buttons.get("b")):
        DRAW_MENU = False

    if DRAW_MENU:
        screen = wt_menu_controls(
            pygame, controller, synth, state, SCREEN_NAMES)
        # if screen is not None:
        SUB_SCREEN = screen
        # DRAW_MENU = False
    else:
        control_f = SUB_SCREENS[SUB_SCREEN][1]

        if control_f is not None:
            synth = control_f(pygame, controller, synth, state)

    return synth
