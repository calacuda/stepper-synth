from stepper_synth_backend import StepperSynthState, StepperSynth
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial
import math


OSC_INDEX = 0
X_INDEX = 0
CONTROLS = [
    "osc-volume",
    "osc-offset",
    "osc-detune",
    "osc-wave-table",
    "osc-power"
]


def draw_volume_dial(pygame, screen, fonts, osc_info, x, y, selected):
    volume = osc_info.volume

    draw_dial(pygame, screen, x, y, volume, selected,
              diameter=SCREEN_WIDTH / 3 / 3 * 0.75)

    draw_text(screen, f"{int(osc_info.volume * 100)} %",
              fonts[1], (x, y), TEXT_COLOR_2)


def mk_triangle(center, radias, thetas):
    return [
        (
            center[0] + -radias * math.cos(math.radians(theta)),
            center[1] + radias * math.sin(math.radians(theta))
        )
        for theta in thetas
    ]


def draw_offset(pygame, screen, fonts, osc_info, top, bottom, left, right, offset_sel):
    offset = osc_info.offset

    center_x = (left + right) / 2
    center_y = (top + bottom) / 2

    height = bottom - top

    draw_text(screen, f"{offset}",
              fonts[0], (center_x, center_y), TEXT_COLOR_1)

    button_h = height / 4 - LINE_WIDTH
    button_w = (right - left) * 0.75 - LINE_WIDTH
    radias = button_h / 4
    button_color = GREEN if offset_sel else TEXT_COLOR_2

    rect = pygame.Rect(
        left, top + height / 8, button_w, button_h)
    rect.centerx = center_x

    pygame.draw.rect(screen, button_color, rect, int(LINE_WIDTH / 2))
    # draw_triangle
    pygame.draw.polygon(screen, button_color, mk_triangle(
        (rect.center[0], rect.center[1] + radias / math.pi), radias,
        [270.0, 270.0 + 120.0, 270.0 - 120.0]))

    rect = pygame.Rect(
        left, top + (height / 8) * 5.125, button_w, button_h)
    rect.centerx = center_x

    pygame.draw.rect(screen, button_color, rect, int(LINE_WIDTH / 2))
    # draw_triangle
    pygame.draw.polygon(screen, button_color, mk_triangle(
        (rect.center[0], rect.center[1] - radias / math.pi), radias,
        [90.0, 90.0 + 120.0, 90.0 - 120.0]))


def draw_detune(pygame, screen, fonts, osc_info, top, bottom, left, right, sel):
    detune = osc_info.detune
    x = (left + right) / 2
    y = (top + bottom) / 2

    draw_dial(pygame, screen, x, y, detune, sel,
              diameter=SCREEN_WIDTH / 3 / 3 * 0.75)

    draw_text(screen, f"{int(detune * 100)} %",
              fonts[1], (x, y), TEXT_COLOR_2)


def draw_wt(pygame, screen, fonts, osc_info, top, bottom, left, right, sel):
    wt = osc_info.wave_table
    offset = LINE_WIDTH
    m_y = (top + bottom) / 2

    pygame.draw.line(screen, LAVENDER, (left + offset, m_y),
                     (right, m_y), int(LINE_WIDTH / 2))
    offset *= 2
    line_color = RED if sel else PEACH
    graph_width = right - left - offset * 2
    x_dist = graph_width / len(wt)
    graph_h = (bottom - top) / 5

    points = [(x_dist * i + left + offset, m_y - s * graph_h)
              for (i, s) in enumerate(wt)]

    pygame.draw.lines(screen, line_color, False,
                      points, width=int(LINE_WIDTH / 2))
    # TODO: add a wave table name display.


def draw_osc(pygame, screen, fonts, synth: StepperSynthState, osc_i, top, bottom, middle_y):
    osc_info = synth.osc[osc_i]
    osc_selected = OSC_INDEX == osc_i
    volume_selected = osc_selected and X_INDEX == 0
    number_section_width = SCREEN_WIDTH / 3

    # displays volume dial
    vol_width = number_section_width / 3
    vol_x = vol_width / 2
    vol_y = middle_y

    draw_volume_dial(pygame, screen, fonts,
                     osc_info, vol_x, vol_y, volume_selected)

    # displays note offset
    offset_left = vol_width
    offset_right = (number_section_width / 3) * 2
    offset_sel = osc_selected and X_INDEX == 1

    draw_offset(pygame, screen, fonts, osc_info, top, bottom,
                offset_left, offset_right, offset_sel)

    # displays detune
    detune_left = offset_right
    detune_right = number_section_width
    detune_sel = osc_selected and X_INDEX == 2

    draw_detune(pygame, screen, fonts, osc_info, top, bottom,
                detune_left, detune_right, detune_sel)

    # Wave table visualizer
    wt_left = detune_right
    wt_right = SCREEN_WIDTH - SCREEN_HEIGHT / 3
    wt_sel = osc_selected and X_INDEX == 3

    draw_wt(pygame, screen, fonts, osc_info, top,
            bottom, wt_left, wt_right, wt_sel)


def draw_osc_menu(pygame, screen, fonts, synth: StepperSynthState):
    third_height = SCREEN_HEIGHT / 3

    for i in range(3):
        top = third_height * i
        bottom = top + third_height
        middle_y = (top + bottom) / 2

        draw_osc(pygame, screen, fonts, synth, i, top, bottom, middle_y)

        if i < 3 - 1:
            pygame.draw.line(screen, GREEN, (0, bottom),
                             (SCREEN_WIDTH, bottom), int(LINE_WIDTH / 2))


def osc_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    return synth
