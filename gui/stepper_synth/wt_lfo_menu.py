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


def format_time(time: float, max: float = 1.0):
    val = max * time

    return f"{val:.4f} sec"


def draw_lfo(pygame, screen, fonts, lfo, top, left, lfo_i):
    # bottom = top + (SCREEN_HEIGHT / 2)
    right = left + (SCREEN_WIDTH / 2)
    w = (SCREEN_WIDTH / 2)
    h = (SCREEN_HEIGHT / 2)
    lfo_sel = (Y_I == lfo_i // 2) and (X_I // 4 == (lfo_i % 2))
    # print(f"{env_i} => {env_sel}")

    # offset = w / 8
    col_w = w / 4
    row_h = h / 4
    row_offset = h / 6
    start_offset = lfo_i // 2

    # display_things = (
    #     (lfo.speed, format_time(lfo.speed)),
    #     # ("D", env.dcy, format_time(env.dcy)),
    #     # ("S", env.sus, f"{round(env.sus * 100)}"),
    #     # ("R", env.rel, format_time(env.rel)),
    # )

    # draw env filter title
    y = row_offset + row_h * ((start_offset + 3) % 4) + top
    x = (left + right) / 2
    color = TEXT_COLOR_2 if not lfo_sel else PEACH
    draw_text(screen, f"LFO {lfo_i + 1}", fonts[0], (x, y), color)

    # for (i, (val, display)) in enumerate(display_things):
    # x = (left + right) / 2

    y = row_offset + row_h * 1 + top
    sel = lfo_sel
    color = TEXT_COLOR_2 if not sel else PEACH
    # draw_text(screen, label, fonts[2], (x, y), color)

    # draw dial
    # y = row_offset + row_h * (start_offset + 1) + top
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
    # move_cursor(controller)
    # return adjust_value(pygame, controller, synth, state)
    return synth
