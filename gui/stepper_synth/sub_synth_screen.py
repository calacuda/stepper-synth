import math
from stepper_synth.utils import set_max
from .config import *
from stepper_synth_backend import GuiParam, Knob, OscType, StepperSynthState, StepperSynth
from .controls import Buttons, buttons


TIMER = 0
ADSR_INDEX = 0
OSC_1_I = 0
OSC_2_I = 0
INDEX = [0, 0, 0, 0, 0]
CONTROLS = [[GuiParam.A, GuiParam.C], [GuiParam.B, GuiParam.D, GuiParam.E], [
    Knob.One, Knob.Two, Knob.Three], [Knob.Five]]


def get_dial_coords(value, center, radias):
    theta = 360 - (150 * value + 15)
    # print("value, theta", value, theta)
    coord = (
        center[0] + -radias * math.cos(math.radians(theta)), center[1] + radias * math.sin(math.radians(theta)))

    return coord


def draw_dial(pygame, screen, value: float, center_x: float, selected: bool):
    diameter = SCREEN_WIDTH / 8
    radias = diameter / 2
    center = (center_x, SCREEN_HEIGHT / 2)

    outline = RED if selected else GREEN

    pygame.draw.circle(screen, outline, center, radias)
    pygame.draw.circle(screen, BACKGROUND_COLOR, center, radias - LINE_WIDTH)

    dial_coord = get_dial_coords(value, center, radias * 1.3)
    min_coord = get_dial_coords(0.0, center, radias * 1.25)
    max_coord = get_dial_coords(1.0, center, radias * 1.25)

    pygame.draw.line(screen, GREEN, center,
                     max_coord, width=LINE_WIDTH)
    pygame.draw.line(screen, GREEN, center,
                     min_coord, width=LINE_WIDTH)
    pygame.draw.line(screen, ACCENT_COLOR, center,
                     dial_coord, width=LINE_WIDTH)


def draw_osc_2_dials(pygame, screen, state: StepperSynthState, left, right):
    x_offset = (right - left) / 3
    x_1 = left + x_offset
    x_2 = x_1 + x_offset
    note_offset = state.gui_params.get(GuiParam.D)
    detune = state.gui_params.get(GuiParam.E)

    # can be removed when detuning is implemented on the backend
    if detune is None:
        detune = 0.0

    draw_dial(pygame, screen, note_offset / 12.0, x_1,
              osc_2_selected() and INDEX[INDEX[4]] == 1)
    draw_dial(pygame, screen, detune, x_2,
              osc_2_selected() and INDEX[INDEX[4]] == 2)


def draw_osc_1_dial(pygame, screen, state: StepperSynthState, left, right):
    center = (left + right) / 2

    draw_dial(pygame, screen, state.gui_params.get(
        GuiParam.C), center, osc_1_selected() and INDEX[INDEX[4]] == 1)


def draw_osc(osc_num: int, pygame, screen, fonts, state: StepperSynthState, osc_type):
    top = 0
    bottom = SCREEN_HEIGHT / 2
    left = (SCREEN_WIDTH / 2) * osc_num
    right = (SCREEN_WIDTH / 2) * (osc_num + 1)
    center_x = (left + right) / 2
    center_y = (top + bottom) / 2

    # print(osc_type)
    color = RED if osc_num == INDEX[4] and INDEX[INDEX[4]
                                                 ] == 0 else TEXT_COLOR_1
    text = fonts[2].render(f'{osc_type}', True, color)
    text_rect = text.get_rect()
    text_rect.center = (center_x, center_y)
    # text_rect.right = SCREEN_WIDTH
    screen.blit(text, text_rect)

    draw_dials = [draw_osc_1_dial, draw_osc_2_dials]

    draw_dials[osc_num](pygame, screen, state, left, right)


def osc_1_selected():
    return INDEX[4] == 0


def osc_2_selected():
    return INDEX[4] == 1


def adsr_selected():
    return INDEX[4] == 2


def low_pass_selected():
    return INDEX[4] == 3


def draw_adsr_graph(pygame, screen, synth_state: StepperSynthState):
    graph_right = SCREEN_WIDTH / 2
    top = SCREEN_HEIGHT / 2 + BOARDER
    bottom = SCREEN_HEIGHT - BOARDER
    left = (((graph_right - BOARDER) / 8) / 2)
    right = graph_right - (((graph_right - BOARDER) / 8) / 2)

    atk = synth_state.knob_params.get(Knob.One)
    dcy = synth_state.knob_params.get(Knob.Two)
    sus = synth_state.knob_params.get(Knob.Three)
    rel = synth_state.knob_params.get(Knob.Four)

    spacing = (right - left) / 4 + left
    # offset = spacing / 2

    origin = (left, bottom)
    a = (spacing * atk + origin[0], top)
    d = (spacing * dcy + a[0], bottom - abs(top - bottom) * sus)
    s = (right - (spacing * rel), d[1])
    r = (right, bottom)

    for (p1, p2) in [(origin, a), (a, d), (d, s), (s, r)]:
        pygame.draw.line(screen, GREEN, p1, p2, width=LINE_WIDTH)

    for i, center in enumerate([a, d, s]):
        # print((x, y))
        border_color = RED if INDEX[INDEX[4]
                                    ] == i and adsr_selected() else POINT_COLOR

        pygame.draw.circle(screen, border_color, center, POINT_DIAMETER)
        pygame.draw.circle(screen, BACKGROUND_COLOR, center,
                           POINT_DIAMETER - LINE_WIDTH)


def draw_low_pass_filter_graph(pygame, screen, synth_state: StepperSynthState):
    graph_right = SCREEN_WIDTH / 2
    top = SCREEN_HEIGHT / 2 + BOARDER
    bottom = SCREEN_HEIGHT - BOARDER
    right = SCREEN_WIDTH - (((graph_right - BOARDER) / 8) / 2)
    left = graph_right + (((graph_right - BOARDER) / 8) / 2)
    width = right - left
    height = bottom - top

    cutoff = synth_state.knob_params.get(Knob.Five)
    res = synth_state.knob_params.get(Knob.Six)

    point = (left + (width * cutoff), top + (height * (1 - res) * 0.5))
    start = (left, bottom - height * 0.5)
    meet = (point[0] - width / 16, bottom - height * 0.5)
    end = (point[0] + width / 8, bottom)

    graph = [(start, meet), (meet, point), (point, end)]

    for (p1, p2) in graph:
        if p2[0] < left:
            continue

        if p1[0] < left:
            slope = (p2[1] - p1[1]) / (p2[0] - p1[0])
            left_inter = (left,
                          slope * (left - p2[0]) + p2[1])
            pygame.draw.line(screen, GREEN, left_inter, p2, width=LINE_WIDTH)
        elif p2[0] > right:
            slope = (p2[1] - p1[1]) / (p2[0] - p1[0])
            right_inter = (right,
                           slope * (right - p1[0]) + p1[1])
            pygame.draw.line(screen, GREEN, p1, right_inter, width=LINE_WIDTH)
            break
        else:
            pygame.draw.line(screen, GREEN, p1, p2, width=LINE_WIDTH)

    border_color = RED if low_pass_selected() else POINT_COLOR

    pygame.draw.circle(screen, border_color, point, POINT_DIAMETER)
    pygame.draw.circle(screen, BACKGROUND_COLOR, point,
                       POINT_DIAMETER - 4)


def draw_divider(pygame, screen, middle_x, middle_y):
    pygame.draw.rect(screen, BACKGROUND_COLOR, pygame.Rect(
        0, middle_y, SCREEN_WIDTH, middle_y))
    pygame.draw.line(screen, GREEN, (middle_x, 0),
                     (middle_x, SCREEN_HEIGHT), width=LINE_WIDTH)
    pygame.draw.line(screen, GREEN, (0, middle_y),
                     (SCREEN_WIDTH, middle_y), width=LINE_WIDTH)


def move_cursor(controller):
    global INDEX

    # if len(controller.pressed_now) > 1:
    if select_mod_pressed(controller):
        # print(controller.pressed_now)
        return

    if controller.is_pressed(buttons.get("a")) and not select_mod_pressed(controller):
        if controller.just_pressed(buttons.get("right")):
            INDEX[4] = (INDEX[4] + 1) % 4
        elif controller.just_pressed(buttons.get("left")):
            INDEX[4] = (INDEX[4] - 1) % 4
        elif controller.just_pressed(buttons.get("up")):
            INDEX[4] = (INDEX[4] + 2) % 4
        elif controller.just_pressed(buttons.get("down")):
            INDEX[4] = (INDEX[4] - 2) % 4
    elif len(controller.pressed_now) == 1 and not select_mod_pressed(controller):
        if controller.just_pressed(buttons.get("right")):
            INDEX[INDEX[4]] += 1
        elif controller.just_pressed(buttons.get("left")):
            INDEX[INDEX[4]] -= 1
        elif controller.just_pressed(buttons.get("up")):
            INDEX[INDEX[4]] -= 1
        elif controller.just_pressed(buttons.get("down")):
            INDEX[INDEX[4]] += 1

        # if len(controller.pressed_now) > 1:
        #     return

        if INDEX[INDEX[4]] >= len(CONTROLS[INDEX[4]]) or INDEX[INDEX[4]] < 0:
            INDEX[INDEX[4]] = 0 if INDEX[INDEX[4]
                                         ] < 0 else len(CONTROLS[INDEX[4]]) - 1

            if controller.just_pressed(buttons.get("right")):
                INDEX[4] = (INDEX[4] + 1) % 4
            elif controller.just_pressed(buttons.get("left")):
                INDEX[4] = (INDEX[4] - 1) % 4
            # elif controller.just_pressed(buttons.get("up")):
            #     INDEX[4] = (INDEX[4] + 2) % 4
            # elif controller.just_pressed(buttons.get("down")):
            #     INDEX[4] = (INDEX[4] - 2) % 4

        # print("index:", INDEX, len(CONTROLS[INDEX[4]]))
    # else:


def timer_is_done(pygame) -> bool:
    return (pygame.time.get_ticks() - TIMER) / 1000 >= 0.1


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, synth_state: StepperSynthState):
    global TIMER

    if not select_mod_pressed(controller) or not timer_is_done(pygame):
        return synth

    new_val = True
    up_pressed = controller.is_pressed(buttons.get("up"))
    down_pressed = controller.is_pressed(buttons.get("down"))
    left_pressed = controller.is_pressed(buttons.get("left"))
    right_pressed = controller.is_pressed(buttons.get("right"))

    if up_pressed and adsr_selected() and INDEX[INDEX[4]] in [1, 2]:
        param = Knob.Three
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif down_pressed and adsr_selected() and INDEX[INDEX[4]] in [1, 2]:
        param = Knob.Three
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif left_pressed and adsr_selected() and INDEX[INDEX[4]] == 1:
        param = Knob.Two
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif right_pressed and adsr_selected() and INDEX[INDEX[4]] == 1:
        param = Knob.Two
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif left_pressed and adsr_selected() and INDEX[INDEX[4]] == 2:
        param = Knob.Four
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif right_pressed and adsr_selected() and INDEX[INDEX[4]] == 2:
        param = Knob.Four
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif left_pressed and adsr_selected() and INDEX[INDEX[4]] == 0:
        param = Knob.One
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif right_pressed and adsr_selected() and INDEX[INDEX[4]] == 0:
        param = Knob.One
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif left_pressed and low_pass_selected():
        param = Knob.Five
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif right_pressed and low_pass_selected():
        param = Knob.Five
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif up_pressed and low_pass_selected():
        param = Knob.Six
        set_to = set_max(synth_state.knob_params.get(param) + 0.01, 1.0)

        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif down_pressed and low_pass_selected():
        param = Knob.Six
        set_to = set_max(synth_state.knob_params.get(param) - 0.01, 1.0)
        # new_val = PythonCmd.SetKnob(param, set_to)
        synth.set_knob_param(param, set_to)
    elif osc_2_selected() and (left_pressed or right_pressed):
        param = CONTROLS[INDEX[4]][INDEX[INDEX[4]]]

        if INDEX[INDEX[4]] == 0:
            # print("setting 0 or 1")
            mod_amt = -1 if left_pressed else 1
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 4.0)
            print(f"setting oscilator to osc-type {set_to}")
            print(f"gui_param => {param}")
        elif INDEX[INDEX[4]] == 1:
            # print("setting 0 or 1")
            mod_amt = -1 if left_pressed else 1
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 12.0)
        elif INDEX[INDEX[4]] == 2:
            mod_amt = -0.01 if left_pressed else 0.01
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 1.0)
        else:
            set_to = 0

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    elif osc_1_selected() and (left_pressed or right_pressed):
        param = CONTROLS[INDEX[4]][INDEX[INDEX[4]]]

        if INDEX[INDEX[4]] == 0:
            mod_amt = -1 if left_pressed else 1
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 4.0)
        elif INDEX[INDEX[4]] == 1:
            mod_amt = -0.01 if left_pressed else 0.01
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 1.0)
        else:
            mod_amt = 0
            set_to = set_max(synth_state.gui_params.get(param) + mod_amt, 1.0)

        # new_val = PythonCmd.SetGuiParam(param, set_to)
        synth.set_gui_param(param, set_to)
    else:
        new_val = False

    if new_val:
        # print(new_val)
        # ipc.send(new_val)
        TIMER = pygame.time.get_ticks()

    return synth


def sub_synth_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    move_cursor(controller)
    return adjust_value(pygame, controller, synth, state)
    # pass


def draw_sub_synth(pygame, screen, fonts, state: StepperSynthState):
    osc_1_type = OscType(state.gui_params.get(GuiParam.A))
    osc_2_type = OscType(state.gui_params.get(GuiParam.B))

    draw_osc(0, pygame, screen, fonts, state, osc_1_type)
    draw_osc(1, pygame, screen, fonts, state, osc_2_type)

    middle_x = SCREEN_WIDTH / 2
    middle_y = SCREEN_HEIGHT / 2

    draw_divider(pygame, screen, middle_x, middle_y)
    draw_adsr_graph(pygame, screen, state)
    draw_low_pass_filter_graph(pygame, screen, state)
