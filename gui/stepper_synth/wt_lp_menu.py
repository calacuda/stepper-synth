from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


LP_INDEX = 0
X_INDEX = 0
CONTROLS = [
    # "osc-volume",
    WTSynthParam.LowPassCutoff,
    # "osc-offset",
    WTSynthParam.LowPassRes,
    # "osc-detune",
    WTSynthParam.LowPassMix,
    # "osc-wave-table",
    WTSynthParam.LowPassTracking,
]
TIMER = 0


def get_x(y, y_1, x, m=2):
    return (y - y_1) / m + x


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
    color = GREEN
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
    sel = LP_INDEX == i and (X_INDEX == 1 or X_INDEX == 0)
    line_color = RED if sel else PEACH

    pygame.draw.lines(screen, line_color, False,
                      points, width=int(LINE_WIDTH / 2))

    x = (SCREEN_WIDTH * (3.5 / 5)) + graph_offset * 2
    y = (top + bottom) / 2
    mix = lp.mix
    sel = LP_INDEX == i and X_INDEX == 2

    draw_dial(pygame, screen, x, y, mix, sel,
              diameter=SCREEN_WIDTH / 5 * 0.75)

    draw_text(screen, f"{int(mix * 100)}%",
              fonts[0], (x, y), TEXT_COLOR_2)

    # draw key track on off button
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
    # move_cursor(controller)
    # return adjust_value(pygame, controller, synth, state)
    return synth
