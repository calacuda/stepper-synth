from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
# from .wt_env_menu import format_time
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


X_I = 0
Y_I = 0
MOVE_TIMER = 0
ADJUST_TIMER = 0


def move_cursor(pygame, controller: Buttons):
    global X_I
    global Y_I
    global MOVE_TIMER

    # print(f"({X_I}, {Y_I})")

    if select_mod_pressed(controller) or not timer_is_done(pygame, MOVE_TIMER, 0.15):
        return

    if controller.is_pressed(buttons.get("up")):
        Y_I -= 1
        Y_I %= 2
    elif controller.is_pressed(buttons.get("down")):
        Y_I += 1
        Y_I %= 2
    elif controller.is_pressed(buttons.get("right")):
        X_I += 1
        X_I %= 2

        if X_I == 0:
            Y_I += 1
            Y_I %= 2
            X_I = 0
    elif controller.is_pressed(buttons.get("left")):
        X_I -= 1
        X_I %= 2

        if X_I == 1:
            Y_I += 1
            Y_I %= 2
            X_I = 1
    else:
        return

    MOVE_TIMER = pygame.time.get_ticks()


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    global ADJUST_TIMER

    if (not select_mod_pressed(controller)) or (not timer_is_done(pygame, ADJUST_TIMER)):
        # TIMER = pygame.time.get_ticks()
        return synth

    lfo_i = (Y_I * 2) + X_I
    lfo = state.lfo[lfo_i]
    amt = lfo.speed
    # print(amt)

    if controller.is_pressed(buttons.get("right")):
        set_to = set_max(amt + 0.05, 1.0, min=0.0)
    elif controller.is_pressed(buttons.get("left")):
        set_to = set_max(amt - 0.05, 1.0, min=0.0)
    else:
        return synth

    # print(f"set_to = {set_to}")

    ADJUST_TIMER = pygame.time.get_ticks()
    synth.wt_param_setter(WTSynthParam.LfoSpeed(lfo_i, set_to))

    return synth


def format_time(time: float, max: float = 1.0):
    val = max * time

    return f"{val:.2f} sec"


def draw_lfo(pygame, screen, fonts, lfo, top, left, lfo_i):
    right = left + (SCREEN_WIDTH / 2)
    w = (SCREEN_WIDTH / 2)
    h = (SCREEN_HEIGHT / 2)
    lfo_sel = (Y_I == lfo_i // 2) and (X_I == (lfo_i % 2))
    col_w = w / 4
    row_h = h / 4
    row_offset = h / 6
    start_offset = lfo_i // 2

    # draw env filter title
    y = row_offset + row_h * ((start_offset + 3) % 4) + top
    x = (left + right) / 2
    color = TEXT_COLOR_2 if not lfo_sel else PEACH
    draw_text(screen, f"LFO {lfo_i + 1}", fonts[0], (x, y), color)

    y = row_offset + row_h * 1 + top
    sel = lfo_sel
    color = TEXT_COLOR_2 if not sel else PEACH

    # draw dial
    draw_dial(pygame, screen, x, y, lfo.speed, sel, diameter=col_w / 2)

    # draw display
    y = row_offset + row_h * 2 + top
    draw_text(screen, format_time(lfo.speed), fonts[1], (x, y), color)


def draw_lfo_menu(pygame, screen, fonts, synth: StepperSynthState):
    lfos = synth.lfo

    # draw horizontal divider
    pygame.draw.line(screen, GREEN, (0, SCREEN_CENTER[1]),
                     (SCREEN_WIDTH, SCREEN_CENTER[1]), int(LINE_WIDTH / 2))
    # draw vertical divider
    pygame.draw.line(screen, GREEN, (SCREEN_CENTER[0], 0),
                     (SCREEN_CENTER[0], SCREEN_WIDTH), int(LINE_WIDTH / 2))

    for i in range(4):
        top = (SCREEN_HEIGHT / 2) * (i // 2)
        left = (SCREEN_WIDTH / 2) * (i % 2)
        draw_lfo(pygame, screen, fonts, lfos[i], top, left, i)


def lfo_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    move_cursor(pygame, controller)
    return adjust_value(pygame, controller, synth, state)
    # return synth
