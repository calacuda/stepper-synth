from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
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
        X_I %= 8

        if X_I == 0:
            Y_I += 1
            Y_I %= 2
            X_I = 0
    elif controller.is_pressed(buttons.get("left")):
        X_I -= 1
        X_I %= 8

        if X_I == 7:
            Y_I += 1
            Y_I %= 2
            X_I = 7
    else:
        return

    MOVE_TIMER = pygame.time.get_ticks()


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    global ADJUST_TIMER

    if (not select_mod_pressed(controller)) or (not timer_is_done(pygame, ADJUST_TIMER)):
        # TIMER = pygame.time.get_ticks()
        return synth

    env_i = (Y_I * 2) + (X_I // 4)
    env = state.adsr[env_i]

    amts = [
        env.atk,
        env.dcy,
        env.sus,
        env.rel,
    ]
    params = [
        WTSynthParam.ADSRAttack,
        WTSynthParam.ADSRDecay,
        WTSynthParam.ADSRSustain,
        WTSynthParam.ADSRRelease,
    ]

    if controller.is_pressed(buttons.get("right")):
        param = params[X_I % 4]
        amt = amts[X_I % 4]
        set_to = set_max(amt + 0.05, 1.0, min=0.0)
    elif controller.is_pressed(buttons.get("left")):
        param = params[X_I % 4]
        amt = amts[X_I % 4]
        set_to = set_max(amt - 0.05, 1.0, min=0.0)
    else:
        return synth

    # print(f"set_to = {set_to}")

    ADJUST_TIMER = pygame.time.get_ticks()
    synth.wt_param_setter(param(env_i, set_to))

    return synth


def format_time(time: float, max: float = 1.0):
    val = max * time

    return f"{val:.3f}"


def draw_env(pygame, screen, fonts, env, top, left, env_i):
    # bottom = top + (SCREEN_HEIGHT / 2)
    right = left + (SCREEN_WIDTH / 2)
    w = (SCREEN_WIDTH / 2)
    h = (SCREEN_HEIGHT / 2)
    env_sel = (Y_I == env_i // 2) and (X_I // 4 == (env_i % 2))
    # print(f"{env_i} => {env_sel}")

    offset = w / 8
    col_w = w / 4
    row_h = h / 4
    row_offset = h / 8
    start_offset = env_i // 2

    display_things = (
        ("A", env.atk, format_time(env.atk)),
        ("D", env.dcy, format_time(env.dcy)),
        ("S", env.sus, f"{round(env.sus * 100)}"),
        ("R", env.rel, format_time(env.rel)),
    )

    # draw env filter title
    y = row_offset + row_h * ((start_offset + 3) % 4) + top
    x = (left + right) / 2
    color = TEXT_COLOR_2 if not env_sel else PEACH
    draw_text(screen, f"ADSR {env_i + 1}", fonts[0], (x, y), color)

    for (i, (label, val, display)) in enumerate(display_things):
        x = offset + col_w * i + left

        y = row_offset + row_h * start_offset + top
        sel = env_sel and X_I % 4 == i
        color = TEXT_COLOR_2 if not sel else PEACH
        draw_text(screen, label, fonts[2], (x, y), color)

        # draw dial
        y = row_offset + row_h * (start_offset + 1) + top
        draw_dial(pygame, screen, x, y, val, sel, diameter=col_w / 2)

        # draw display
        y = row_offset + row_h * (start_offset + 2) + top
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
    move_cursor(pygame, controller)
    return adjust_value(pygame, controller, synth, state)
    # return synth
