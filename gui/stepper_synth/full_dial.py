# from stepper_synth_backend import GuiParam, Knob, SynthEngineType, StepperSynth, StepperSynthState, Screen, EffectType
from .config import *
from .utils import *
import math


def get_dial_coords(value, center, radias):
    dead_zone = 22.5
    theta = ((360 - dead_zone) * (1.0 - value) + (90 + dead_zone / 2))
    # print("value, theta", value, theta)
    coord = (
        center[0] + -radias * math.cos(math.radians(theta)), center[1] + radias * math.sin(math.radians(theta)))

    return coord


def scale_by(point, scale, center):
    x = point[0] - center[0]
    x = x * scale + center[0]
    y = point[1] - center[1]
    y = y * scale + center[1]

    return (x, y)


def draw_dial(pygame, screen, x, y, value, selected):
    diameter = SCREEN_WIDTH / 8
    radias = diameter / 2
    center = (x, y)

    outline = RED if selected else GREEN

    pygame.draw.circle(screen, outline, center, radias)
    pygame.draw.circle(screen, BACKGROUND_COLOR, center, radias - LINE_WIDTH)

    dial_coord = get_dial_coords(value, center, radias)
    min_coord = get_dial_coords(0.0, center, radias)
    max_coord = get_dial_coords(1.0, center, radias)

    pygame.draw.line(screen, GREEN, scale_by(max_coord, 1.5, center),
                     max_coord, width=LINE_WIDTH*2)
    pygame.draw.line(screen, GREEN, scale_by(min_coord, 1.5, center),
                     min_coord, width=LINE_WIDTH*2)

    n_steps = 12

    for i in range(1, n_steps):
        notch_value = i / n_steps
        # print()
        p1 = get_dial_coords(notch_value, center, radias)
        p2 = scale_by(p1, 1.25, center)
        pygame.draw.line(screen, GREEN, p1,
                         p2, width=LINE_WIDTH)

    pygame.draw.line(screen, ACCENT_COLOR, center,
                     dial_coord, width=LINE_WIDTH)
