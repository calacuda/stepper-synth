from .controls import Buttons, buttons
from stepper_synth_backend import State, GuiParam, Knob, PythonCmd
from .config import *
import math

vert_middle = SCREEN_HEIGHT / 2
SPEAKER_CENTER = (SCREEN_WIDTH * (3 / 4), vert_middle)
LAST_TICK_TIME = 0
LAST_THETA = 0.0
SPEAKER_RAD = SCREEN_WIDTH / 5
GRAPH_RIGHT = SCREEN_WIDTH - \
    (SCREEN_WIDTH - SPEAKER_CENTER[0]) - SPEAKER_RAD - BOARDER
CONTROLS = (
    [Knob.One, Knob.Two, Knob.Three, Knob.Four,
     Knob.Five, Knob.Six, Knob.Seven, Knob.Eight],
    [GuiParam.A, GuiParam.B, GuiParam.C, GuiParam.D]
)
INDEX = [0, 0, 0]
TIMER = 0


def draw_bg(pygame, screen):
    # draw horizontal line
    pygame.draw.line(screen, GREEN,
                     (0, vert_middle), (SCREEN_WIDTH, vert_middle), width=4)
    # draw speaker circle
    pygame.draw.circle(screen, GREEN, SPEAKER_CENTER, SPEAKER_RAD)

    pygame.draw.circle(screen, BACKGROUND_COLOR,
                       SPEAKER_CENTER, SPEAKER_RAD - 4)


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
        SPEAKER_CENTER[0] + SPEAKER_RAD * math.cos(theta), SPEAKER_CENTER[1] + SPEAKER_RAD * math.sin(theta))

    # print(coord)

    # draw the line
    pygame.draw.line(screen, GREEN,
                     SPEAKER_CENTER, coord, width=4)

    LAST_THETA += (2.0 * math.pi * line_speed * seconds)
    LAST_THETA %= (2 * math.pi)
    LAST_TICK_TIME = ticks


def draw_draw_bar_level(screen, fonts, bar_val: float, center_x: float, selected: bool):
    bar_level = str(round(bar_val * 8))
    text_color = RED if selected else TEXT_COLOR_1
    font = fonts[1]
    display = font.render(
        bar_level, True, text_color)
    text_rect = display.get_rect()
    (_, _, _, text_height) = text_rect

    x, y = (center_x, (BOARDER + (text_height / 2)))

    text_rect.center = (x, y)

    screen.blit(display, text_rect)

    return text_rect.bottom


def draw_draw_bar_line(pygame, screen, fonts, bar_val: float, center_x: float, level_lable_bottom: float):
    bottom = SCREEN_HEIGHT / 2 - BOARDER
    top = level_lable_bottom + BOARDER
    level_marker = bottom - ((bottom - top) * bar_val)
    width = 4

    # draw full line
    pygame.draw.line(screen, SURFACE_1,
                     (center_x, top), (center_x, level_marker), width=width)
    # draw level line
    pygame.draw.line(screen, GREEN,
                     (center_x, level_marker), (center_x, bottom), width=width)

    # draw indicator circle
    pygame.draw.circle(
        screen, POINT_COLOR, (center_x, level_marker), POINT_DIAMETER)

    pygame.draw.circle(screen, BACKGROUND_COLOR,
                       (center_x, level_marker), POINT_DIAMETER - width)


def draw_draw_bar(pygame, screen, fonts, bar_val: float, center_x: float, selected: bool):
    level_lable_bottom = draw_draw_bar_level(
        screen, fonts, bar_val, center_x, selected)
    draw_draw_bar_line(pygame, screen, fonts, bar_val,
                       center_x, level_lable_bottom)


def draw_draw_bars(pygame, screen, fonts, synth_state: State):
    draw_bar_values = [
        synth_state.knob_params.get(Knob.One),
        synth_state.knob_params.get(Knob.Two),
        synth_state.knob_params.get(Knob.Three),
        synth_state.knob_params.get(Knob.Four),
        synth_state.knob_params.get(Knob.Five),
        synth_state.knob_params.get(Knob.Six),
        synth_state.knob_params.get(Knob.Seven),
        synth_state.knob_params.get(Knob.Eight),
    ]

    # spacing = (SCREEN_WIDTH -
    #            SPEAKER_CENTER[0] - (SPEAKER_RAD / 2)) * 0.4
    spacing = (GRAPH_RIGHT - BOARDER) / 8
    offset = spacing / 2 + (BOARDER * 2)

    for (i, bar_val) in enumerate(draw_bar_values):
        center_x = offset + (spacing * i)
        selected = INDEX[2] == 0 and INDEX[INDEX[2]] == i
        draw_draw_bar(pygame, screen, fonts, bar_val, center_x, selected)


def draw_adsr_graph(pygame, screen, synth_state: State):
    top = SCREEN_HEIGHT / 2 + BOARDER
    bottom = SCREEN_HEIGHT - BOARDER
    left = BOARDER
    right = GRAPH_RIGHT - (((GRAPH_RIGHT - BOARDER) / 8) / 2)

    atk = synth_state.gui_params.get(GuiParam.A)
    dcy = synth_state.gui_params.get(GuiParam.B)
    sus = synth_state.gui_params.get(GuiParam.C)
    rel = synth_state.gui_params.get(GuiParam.D)

    spacing = (right - left) / 4 + left
    offset = spacing / 2

    origin = (left + offset, bottom)
    a = (spacing * atk + origin[0], top)
    d = (spacing * dcy + a[0], abs(top - bottom) * sus + top)
    s = (right - (spacing * rel), d[1])
    r = (right, bottom)

    for (p1, p2) in [(origin, a), (a, d), (d, s), (s, r)]:
        pygame.draw.line(screen, GREEN, p1, p2, width=4)

    for i, center in enumerate([a, d, s, r]):
        # print((x, y))
        border_color = RED if INDEX[2] == 1 and INDEX[INDEX[2]
                                                      ] == i else POINT_COLOR

        pygame.draw.circle(screen, border_color, center, POINT_DIAMETER)
        pygame.draw.circle(screen, BACKGROUND_COLOR, center,
                           POINT_DIAMETER - 4)


def move_cursor(controller):
    global INDEX

    if len(controller.pressed_now) > 1:
        return

    if controller.just_pressed(buttons.get("right")):
        max_len = len(CONTROLS[INDEX[2]])

        INDEX[INDEX[2]] += 1
        INDEX[INDEX[2]] %= max_len
    elif controller.just_pressed(buttons.get("left")):
        max_len = len(CONTROLS[INDEX[2]])

        INDEX[INDEX[2]] -= 1
        INDEX[INDEX[2]] %= max_len
    elif controller.just_pressed(buttons.get("up")) or controller.just_pressed(buttons.get("down")):
        INDEX[2] += 1
        INDEX[2] %= len(CONTROLS)


def timer_is_done(pygame) -> bool:
    return (pygame.time.get_ticks() - TIMER) / 1000 >= 0.1


def adjust_value(pygame, controller: Buttons, ipc, synth_state: State):
    global TIMER

    if not controller.is_pressed(buttons.get("a")):
        return

    # print(INDEX)
    param = CONTROLS[INDEX[2]][INDEX[INDEX[2]]]
    new_val = None

    if INDEX[2] == 1 and controller.is_pressed(buttons.get("right")) and timer_is_done(pygame):
        # TIMER = 20
        new_val = PythonCmd.SetGuiParam(
            param, (synth_state.gui_params.get(param) + 0.01) % 1.0)
    elif INDEX[2] == 1 and controller.is_pressed(buttons.get("left")) and timer_is_done(pygame):
        # TIMER = 20
        new_val = PythonCmd.SetGuiParam(
            param, (synth_state.gui_params.get(param) - 0.01) % 1.0)
    elif INDEX[2] == 0 and controller.is_pressed(buttons.get("up")) and timer_is_done(pygame):
        new_val = PythonCmd.SetKnob(
            param, (synth_state.knob_params.get(param) + 0.05) % 1.0)
    elif INDEX[2] == 0 and controller.is_pressed(buttons.get("down")) and timer_is_done(pygame):
        new_val = PythonCmd.SetKnob(
            param, (synth_state.knob_params.get(param) - 0.05) % 1.0)

    if new_val is not None:
        # print(new_val)
        ipc.send(new_val)
        TIMER = pygame.time.get_ticks()


def organ_controls(pygame, controller: Buttons, ipc, synth_state: State):
    move_cursor(controller)
    adjust_value(pygame, controller, ipc, synth_state)


def draw_organ(pygame, screen, fonts, synth_state: State):
    draw_bg(pygame, screen)
    draw_speaker(pygame, screen, synth_state)
    draw_draw_bars(pygame, screen, fonts, synth_state)
    draw_adsr_graph(pygame, screen, synth_state)
