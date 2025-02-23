from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


X_I = 0
Y_I = 0


def format_time(time: float, max: float = 1.0):
    val = max * time

    return f"{val:.3f}"


def draw_env(pygame, screen, fonts, env, top, left, env_i):
    bottom = top + (SCREEN_HEIGHT / 2)
    right = left + (SCREEN_WIDTH / 2)
    w = (SCREEN_WIDTH / 2)
    h = (SCREEN_HEIGHT / 2)
    env_sel = (Y_I <= env_i // 2) and (X_I // 4 == env_i)

    offset = w / 8
    col_w = w / 4
    row_h = h / 3
    row_offset = h / 6

    display_things = (
        (f"A-{env_i + 1}", env.atk, format_time(env.atk)),
        ("D", env.dcy, format_time(env.dcy)),
        ("S", env.sus, f"{round(env.sus * 100)}"),
        ("R", env.rel, format_time(env.rel)),
    )

    # print(env_i, env_sel, f"{X_I} => {X_I % 4}")

    for (i, (label, val, display)) in enumerate(display_things):
        x = offset + col_w * i + left

        y = row_offset + row_h * 0 + top
        sel = env_sel and X_I % 4 == i
        color = TEXT_COLOR_2 if not sel else PEACH
        draw_text(screen, label, fonts[2], (x, y), color)

        # TODO: draw dial
        y = row_offset + row_h * 1 + top
        draw_dial(pygame, screen, x, y, val, sel, diameter=col_w / 2)

        # TODO: draw display
        y = row_offset + row_h * 2 + top
        draw_text(screen, display, fonts[1], (x, y), color)


def draw_env_menu(pygame, screen, fonts, synth: StepperSynthState):
    envs = synth.adsr

    # draw horizontal divider
    pygame.draw.line(screen, GREEN, (0, SCREEN_CENTER[1]),
                     (SCREEN_WIDTH, SCREEN_CENTER[1]), int(LINE_WIDTH / 2))
    # draw vertical divider
    pygame.draw.line(screen, GREEN, (SCREEN_CENTER[0], 0),
                     (SCREEN_CENTER[0], SCREEN_WIDTH), int(LINE_WIDTH / 2))

    for i in range(4):
        top = (SCREEN_HEIGHT / 2) * (i // 2)
        left = (SCREEN_WIDTH / 2) * (i % 2)
        draw_env(pygame, screen, fonts, envs[i], top, left, i)


def env_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    # move_cursor(controller)
    # return adjust_value(pygame, controller, synth, state)
    return synth
