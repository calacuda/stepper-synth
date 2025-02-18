from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


CURSOR = 0


def draw_wt_menu(pygame, screen, fonts, screens):
    menu_w = SCREEN_WIDTH * 0.75
    menu_h = SCREEN_HEIGHT * 0.2
    rect = pygame.Rect(
        0, 0, menu_w, menu_h)
    rect.center = SCREEN_CENTER
    pygame.draw.rect(screen, BACKGROUND_COLOR, rect)
    color = GREEN
    pygame.draw.rect(screen, color, rect, LINE_WIDTH)
    offset = (SCREEN_WIDTH - menu_w) / 2
    box = menu_w / len(screens)

    for i in range(len(screens)):
        x = offset + box * i + box / 2
        color = TEXT_COLOR_1 if i == CURSOR else TEXT_COLOR_2

        draw_diagonal_text(screen, screens[i],
                           fonts[1], (x, SCREEN_HEIGHT / 2), color)


def wt_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState, screens):
    global CURSOR

    # if select_mod_pressed(controller):
    #     return

    if controller.just_pressed(buttons.get("right")):
        CURSOR += 1
        CURSOR %= len(screens)
    elif controller.just_pressed(buttons.get("left")):
        CURSOR -= 1
        CURSOR %= len(screens)
    elif controller.just_pressed(buttons.get("a")):
        return CURSOR

    return CURSOR
