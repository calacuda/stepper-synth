from .controls import Buttons, buttons
from stepper_synth_backend import GuiParam, Knob, StepperSynthState, StepperSynth
from .config import *
from .utils import *
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
    [GuiParam.A, GuiParam.B, GuiParam.C]
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


def draw_speaker(pygame, screen, state: StepperSynthState):
    global LAST_TICK_TIME
    global LAST_THETA

    line_speed = state.gui_params.get(GuiParam.E) * 0.5

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


def draw_draw_bar_line(pygame, screen, bar_val: float, center_x: float, level_lable_bottom: float, selected: bool):
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
    border_color = RED if selected else POINT_COLOR

    pygame.draw.circle(
        screen, border_color, (center_x, level_marker), POINT_DIAMETER)

    pygame.draw.circle(screen, BACKGROUND_COLOR,
                       (center_x, level_marker), POINT_DIAMETER - width)


def draw_draw_bar(pygame, screen, fonts, bar_val: float, center_x: float, selected: bool):
    level_lable_bottom = draw_draw_bar_level(
        screen, fonts, bar_val, center_x, selected)
    draw_draw_bar_line(pygame, screen, bar_val,
                       center_x, level_lable_bottom, selected)


def draw_draw_bars(pygame, screen, fonts, state: StepperSynthState):
    draw_bar_values = [
        state.knob_params.get(Knob.One),
        state.knob_params.get(Knob.Two),
        state.knob_params.get(Knob.Three),
        state.knob_params.get(Knob.Four),
        state.knob_params.get(Knob.Five),
        state.knob_params.get(Knob.Six),
        state.knob_params.get(Knob.Seven),
        state.knob_params.get(Knob.Eight),
    ]

    # spacing = (SCREEN_WIDTH -
    #            SPEAKER_CENTER[0] - (SPEAKER_RAD / 2)) * 0.4
    spacing = (GRAPH_RIGHT - BOARDER) / 8
    offset = spacing / 2 + (BOARDER * 2)

    for (i, bar_val) in enumerate(draw_bar_values):
        center_x = offset + (spacing * i)
        selected = INDEX[2] == 0 and INDEX[INDEX[2]] == i
        draw_draw_bar(pygame, screen, fonts, bar_val, center_x, selected)


def draw_adsr_graph(pygame, screen, state: StepperSynthState):
    top = SCREEN_HEIGHT / 2 + BOARDER
    bottom = SCREEN_HEIGHT - BOARDER
    left = BOARDER
    right = GRAPH_RIGHT - (((GRAPH_RIGHT - BOARDER) / 8) / 2)

    atk = state.gui_params.get(GuiParam.A)
    dcy = state.gui_params.get(GuiParam.B)
    sus = state.gui_params.get(GuiParam.C)
    rel = state.gui_params.get(GuiParam.D)

    spacing = (right - left) / 4 + left
    offset = spacing / 2

    origin = (left + offset, bottom)
    a = (spacing * atk + origin[0], top)
    d = (spacing * dcy + a[0], bottom - abs(top - bottom) * sus)
    s = (right - (spacing * rel), d[1])
    r = (right, bottom)

    for (p1, p2) in [(origin, a), (a, d), (d, s), (s, r)]:
        pygame.draw.line(screen, GREEN, p1, p2, width=4)

    for i, center in enumerate([a, d, s]):
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
    elif controller.just_pressed(buttons.get("up")):
        # INDEX[2] += 1
        # INDEX[2] %= len(CONTROLS)
        INDEX[2] = int(set_max(INDEX[2] - 1, float(len(CONTROLS))))
    elif controller.just_pressed(buttons.get("down")):
        # INDEX[2] += 1
        # INDEX[2] %= len(CONTROLS)
        INDEX[2] = int(set_max(INDEX[2] + 1, float(len(CONTROLS))))


def timer_is_done(pygame) -> bool:
    return (pygame.time.get_ticks() - TIMER) / 1000 >= 0.1


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, synth_state: StepperSynthState):
    global TIMER

    if not select_mod_pressed(controller):
        return synth

    # print(INDEX)
    param = CONTROLS[INDEX[2]][INDEX[INDEX[2]]]
    new_val = None
    up_pressed = controller.is_pressed(buttons.get("up"))
    down_pressed = controller.is_pressed(buttons.get("down"))
    left_pressed = controller.is_pressed(buttons.get("left"))
    right_pressed = controller.is_pressed(buttons.get("right"))
    adr = [GuiParam.A, GuiParam.B, GuiParam.D]
    ds = [GuiParam.B, GuiParam.C]
    timer_done = timer_is_done(pygame)
    gui_param = INDEX[2] == 1
    knob_param = INDEX[2] == 0
    new_val = True

    if gui_param and right_pressed and timer_done and param in adr:
        set_to = set_max(synth_state.gui_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif gui_param and left_pressed and timer_done and param in adr:
        set_to = set_max(synth_state.gui_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif gui_param and left_pressed and timer_done and param == GuiParam.C:
        param = GuiParam.D
        set_to = set_max(synth_state.gui_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif gui_param and right_pressed and timer_done and param == GuiParam.C:
        param = GuiParam.D
        set_to = set_max(synth_state.gui_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif gui_param and up_pressed and timer_done and param in ds:
        param = GuiParam.C
        set_to = set_max(synth_state.gui_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif gui_param and down_pressed and timer_done and param in ds:
        param = GuiParam.C
        set_to = set_max(synth_state.gui_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif knob_param and up_pressed and timer_done:
        set_to = set_max(synth_state.knob_params.get(param) + 0.05, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif knob_param and down_pressed and timer_done:
        set_to = set_max(synth_state.knob_params.get(param) - 0.05, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    else:
        new_val = False

    if new_val:
        # print(new_val)
        # ipc.send(new_val)
        TIMER = pygame.time.get_ticks()

    return synth


def organ_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    move_cursor(controller)
    return adjust_value(pygame, controller, synth, state)


def draw_organ(pygame, screen, fonts, state: StepperSynthState):
    draw_bg(pygame, screen)
    draw_speaker(pygame, screen, state)
    draw_draw_bars(pygame, screen, fonts, state)
    draw_adsr_graph(pygame, screen, state)
