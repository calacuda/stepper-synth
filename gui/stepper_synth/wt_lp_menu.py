from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


LP_INDEX = 0
X_INDEX = 0
# CONTROLS = [
#     # "osc-volume",
#     WTSynthParam.LowPassCutoff,
#     # "osc-offset",
#     WTSynthParam.LowPassRes,
#     # "osc-detune",
#     WTSynthParam.LowPassMix,
#     # "osc-wave-table",
#     WTSynthParam.LowPassTracking,
# ]
N_COL = 3
TIMER = 0


def get_x(y, y_1, x, m=2):
    return (y - y_1) / m + x


def move_cursor(controller: Buttons):
    global LP_INDEX
    global X_INDEX
    global TIMER

    if select_mod_pressed(controller):
        return

    if controller.just_pressed(buttons.get("up")):
        LP_INDEX -= 1
        LP_INDEX %= 2
    elif controller.just_pressed(buttons.get("down")):
        LP_INDEX += 1
        LP_INDEX %= 2
    elif controller.just_pressed(buttons.get("right")):
        X_INDEX += 1
        X_INDEX %= N_COL
    elif controller.just_pressed(buttons.get("left")):
        X_INDEX -= 1
        X_INDEX %= N_COL
    # else:
    #     print(X_INDEX)


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    global TIMER

    if (not select_mod_pressed(controller)) or (not timer_is_done(pygame, TIMER)):
        # TIMER = pygame.time.get_ticks()
        return synth

    filter = state.filter[LP_INDEX]
    amts = [
        filter.cutoff,
        filter.mix,
        filter.keytracking,
        filter.res,
    ]
    params = [
        WTSynthParam.LowPassCutoff,
        WTSynthParam.LowPassMix,
        WTSynthParam.LowPassTracking,
        WTSynthParam.LowPassRes,
    ]

    if (controller.is_pressed(buttons.get("left")) or controller.is_pressed(buttons.get("right"))) and X_INDEX == 2:
        # control = WTSynthParam.LowPassRes
        control = params[2]
        set_to = not amts[2]
    elif controller.is_pressed(buttons.get("right")):
        # set_to = GETTERS[X_INDEX](amts, True)
        control = params[X_INDEX]
        set_to = amts[X_INDEX] + 0.05
        set_to = set_max(set_to, 1.0, min=0.0)
    elif controller.is_pressed(buttons.get("left")):
        # set_to = set_max(amts[X_INDEX] - 0.05, 1.0, min=0.0)
        # set_to = GETTERS[X_INDEX](amts, False)
        control = params[X_INDEX]
        set_to = amts[X_INDEX] - 0.05
        set_to = set_max(set_to, 1.0, min=0.0)
    elif controller.is_pressed(buttons.get("up")) and X_INDEX == 0:
        # control = WTSynthParam.LowPassRes
        control = params[3]
        set_to = amts[3] + 0.05
        set_to = set_max(set_to, 1.0, min=0.0)
    elif controller.is_pressed(buttons.get("down")) and X_INDEX == 0:
        # control = WTSynthParam.LowPassRes
        control = params[3]
        set_to = amts[3] - 0.05
        set_to = set_max(set_to, 1.0, min=0.0)
    else:
        return synth

    # print(f"set_to = {set_to}")

    TIMER = pygame.time.get_ticks()
    synth.wt_param_setter(control(LP_INDEX, set_to))

    return synth


def draw_lp(pygame, screen, fonts, synth: StepperSynthState, top: int):
    i = 0 if not top else 1
    lp = synth.filter[i]
    bottom = top + SCREEN_CENTER[1]
    height = SCREEN_CENTER[1]
    graph_h = height * 0.8
    graph_w = SCREEN_WIDTH * (3/5)
    m_y = (top + bottom) / 2
    cutoff = lp.cutoff
    # cutoff = 0.75
    res = lp.res
    # res = 1

    graph_offset = SCREEN_WIDTH * (.125/5)
    rect = pygame.Rect(0, 0, graph_w, graph_h)
    rect.center = (SCREEN_WIDTH * (1.5/5) + graph_offset, m_y)
    sel = LP_INDEX == i and X_INDEX == 0
    color = RED if sel else GREEN
    pygame.draw.rect(screen, color, rect, LINE_WIDTH)

    # draw graph
    vert = (graph_offset + graph_w * cutoff, m_y - graph_h * 0.5)
    p_1 = (graph_offset, m_y)
    p_2 = (get_x(0, graph_h / 2 * res, vert[0]), m_y)
    y = m_y - (graph_h / 2 * res)
    p_3 = (p_2[0] + (vert[0] - p_2[0]), y)
    p_4 = (vert[0] + (vert[0] - p_3[0]), y)
    # p_4 = (vert[0] + graph_h * 0.25, y)
    p_5 = (get_x((graph_h / 2) + graph_h / 2,
                 (y - top), p_4[0]), m_y + graph_h / 2)
    if p_5[0] > rect.left + rect.width:
        y = 2 * ((rect.left + rect.width) - p_4[0]) + p_4[1]
        # ((graph_h / 2) + graph_h / 2,
        #           (y - top), p_4[0])
        p_5 = (rect.left + rect.width, y)

    # p_5 = (, m_y + graph_h / 2)
    points = [p_1, p_2, p_3, p_4, p_5]
    # points = [p_1, p_2, p_3, p_4]
    # points = [p for p in points if p[0] < rect.left + rect.width]
    line_color = PEACH

    pygame.draw.lines(screen, line_color, False,
                      points, width=int(LINE_WIDTH / 2))

    x = (SCREEN_WIDTH * (3.5 / 5)) + graph_offset * 2
    y = (top + bottom) / 2
    mix = lp.mix
    sel = LP_INDEX == i and X_INDEX == 1

    draw_dial(pygame, screen, x, y, mix, sel,
              diameter=SCREEN_WIDTH / 5 * 0.75)

    draw_text(screen, f"{int(mix * 100)}%",
              fonts[0], (x, y), TEXT_COLOR_2)

    # draw key track on off button
    sel = LP_INDEX == i and X_INDEX == 2
    m_x = (SCREEN_WIDTH * (4.35 / 5)) + graph_offset * 2
    # m_y =
    outer_r = SCREEN_WIDTH / 5 * 0.25
    inner_r = outer_r - LINE_WIDTH * 2
    outline = TEXT_COLOR_2 if not sel else RED
    inner = GREEN if lp.keytracking else TEXT_COLOR_2

    pygame.draw.circle(screen, outline, (m_x, m_y), outer_r)
    pygame.draw.circle(screen, inner, (m_x, m_y), inner_r)


def draw_lp_menu(pygame, screen, fonts, synth: StepperSynthState):
    pygame.draw.line(screen, GREEN, (0, SCREEN_CENTER[1]),
                     (SCREEN_WIDTH, SCREEN_CENTER[1]), int(LINE_WIDTH / 2))

    draw_lp(pygame, screen, fonts, synth, 0)
    draw_lp(pygame, screen, fonts, synth, int(SCREEN_CENTER[1]))


def lp_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    move_cursor(controller)
    return adjust_value(pygame, controller, synth, state)
    # return synth
