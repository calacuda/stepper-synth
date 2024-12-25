from .controls import Buttons, buttons
from stepper_synth_backend import GuiParam, Knob, StepperSynthState, StepperSynth
from .config import *
from .utils import *
from .full_dial import draw_dial


INDEX = 0
CONTROLS = [Knob.Four, Knob.Three, Knob.Two, Knob.One]
TIMER = 0


def move_cursor(controller: Buttons):
    global INDEX

    if select_mod_pressed(controller):
        return

    if controller.just_pressed(buttons.get("right")):
        # print(INDEX[2])
        max_len = len(CONTROLS)

        INDEX += 1
        INDEX %= max_len
    elif controller.just_pressed(buttons.get("left")):
        # print(INDEX[2])
        max_len = len(CONTROLS)

        INDEX -= 1
        INDEX %= max_len
    elif controller.just_pressed(buttons.get("up")):
        # INDEX[2] += 1
        # INDEX[2] %= len(CONTROLS)
        # INDEX[2] = int(
        #     set_max(INDEX[2] - 1, float(len(CONTROLS)) - 1.0, min=0.0))
        max_len = len(CONTROLS)

        INDEX -= 2
        INDEX %= max_len
    elif controller.just_pressed(buttons.get("down")):
        # INDEX[2] += 1
        # INDEX[2] %= len(CONTROLS)
        # INDEX[max_len = len(CONTROLS[INDEX[2]])
        max_len = len(CONTROLS)
        INDEX += 2
        INDEX %= max_len


def timer_is_done(pygame) -> bool:
    return (pygame.time.get_ticks() - TIMER) / 1000 >= 0.1


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    global TIMER

    if controller.just_pressed(buttons.get("a")):
        synth.toggle_effect_power()

    if (not select_mod_pressed(controller)) or (not timer_is_done(pygame)):
        # TIMER = pygame.time.get_ticks()
        return synth

    param = CONTROLS[INDEX]
    get_param = list(state.params.keys())
    get_param.sort()
    get_param = get_param[INDEX]

    if controller.is_pressed(buttons.get("right")):
        set_to = set_max(state.params.get(get_param) + 0.01, 1.0)
    elif controller.is_pressed(buttons.get("left")):
        set_to = set_max(state.params.get(get_param) - 0.01, 1.0, min=0.0)
    else:
        return synth

    # print(f"set_to = {set_to}")

    TIMER = pygame.time.get_ticks()
    synth.set_knob_param(param, set_to)

    return synth


def draw_reverb_menu(pygame, screen, fonts, state: StepperSynthState):
    # gain = state.params.get("Gain")
    # decay = state.params.get("Decay")
    #  = state.params.get("Damping")
    # gain = state.params.get("Gain")
    params = list(state.params.items())

    params.sort(key=lambda pair: pair[0])

    x_offset = SCREEN_WIDTH / 4.0
    y_offset = SCREEN_HEIGHT / 4.0

    # print(params)

    for (i, (key, value)) in enumerate(params):
        x = x_offset + ((SCREEN_WIDTH / 2.0) * (i % 2))
        y = y_offset + ((SCREEN_HEIGHT / 2.0) * (i // 2))
        draw_dial(pygame, screen, x, y, value, i == INDEX)

        color = RED if i == INDEX else TEXT_COLOR_1

        if not i % 2:
            text = f"< {key}"
        else:
            text = f"{key} >"
        display = fonts[2].render(
            text, True, color)
        text_rect = display.get_rect()

        if not i % 2:
            text_rect.center = (int(x + x_offset), int(y))
            text_rect.bottom = y
        else:
            text_rect.center = (int(x - x_offset), int(y))
            text_rect.top = y

        screen.blit(display, text_rect)

    if not state.effect_on:
        # put a pulsing "Off" icon in the middle of the screen
        # height = SCREEN_HEIGHT / 4
        # width = SCREEN_WIDTH / 4
        x = SCREEN_WIDTH / 2
        y = SCREEN_HEIGHT / 2

        color = RED
        display = fonts[2].render(
            "OFF", True, color)
        text_rect = display.get_rect()
        text_rect.center = (x, y)
        screen.blit(display, text_rect)


def reverb_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    move_cursor(controller)
    return adjust_value(pygame, controller, synth, state)
