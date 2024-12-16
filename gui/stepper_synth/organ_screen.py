from stepper_synth_backend import State, GuiParam
from .config import *
import math

vert_middle = SCREEN_HEIGHT / 2
SPEAKER_CENTER = (SCREEN_WIDTH * 3 / 4, vert_middle)
LAST_TICK_TIME = 0
LAST_THETA = 0.0


def draw_bg(pygame, screen):
    # draw horizontal line
    pygame.draw.line(screen, GREEN,
                     (0, vert_middle), (SCREEN_WIDTH, vert_middle), width=4)
    # draw speaker circle
    pygame.draw.circle(screen, GREEN, SPEAKER_CENTER, SCREEN_WIDTH / 5)

    pygame.draw.circle(screen, BACKGROUND_COLOR,
                       SPEAKER_CENTER, (SCREEN_WIDTH / 5) - 4)


def draw_speaker(pygame, screen, synth_state: State):
    global LAST_TICK_TIME
    global LAST_THETA

    line_speed = (400.0 * synth_state.gui_params.get(GuiParam.E)) / 60.0

    # claculate time since last update
    ticks = pygame.time.get_ticks()
    seconds = (ticks-LAST_TICK_TIME)/1000

    # if not seconds:
    #     return

    theta = LAST_THETA

    # claculate line posisiton
    coord = (
        SPEAKER_CENTER[0] + ((SCREEN_WIDTH / 6) - 4) * math.cos(theta), SPEAKER_CENTER[1] + ((SCREEN_WIDTH / 6) - 4) * math.sin(theta))

    # print(coord)

    # draw the line
    pygame.draw.line(screen, GREEN,
                     SPEAKER_CENTER, coord, width=4)

    LAST_THETA += (2.0 * math.pi * line_speed * seconds)
    LAST_THETA %= (2 * math.pi)
    LAST_TICK_TIME = ticks


def draw_organ(pygame, screen, synth_state: State):
    draw_bg(pygame, screen)
    draw_speaker(pygame, screen, synth_state)
