from .controls import Buttons, buttons
from stepper_synth_backend import StepperSynthState, StepperSynth, StepCmd, Screen
from .config import *
from .utils import *
from dataclasses import dataclass


@dataclass()
class Key:
    note: int
    white_key: bool


KEYS = [
    Key(0, True),
    Key(1, False),
    Key(2, True),
    Key(3, False),
    Key(4, True),
    Key(5, True),
    Key(6, False),
    Key(7, True),
    Key(8, False),
    Key(9, True),
    Key(10, False),
    Key(11, True),
]


def draw_octave(pygame, screen, state: StepperSynthState, top: float, width: float, offset: float, octave_n: int, playing):
    key_width = width / 7
    half_width = (key_width) * 0.5
    bottom = SCREEN_HEIGHT - LINE_WIDTH
    w_height = SCREEN_HEIGHT - top - LINE_WIDTH
    b_height = (SCREEN_HEIGHT - top) * 0.5 - LINE_WIDTH
    white_keys_found = 0
    black_keys_found = 1
    b_width = width / 12

    # for (i, key) in enumerate([key for key in KEYS if key.white_key]):
    for key in KEYS:
        if key.white_key:
            left = key_width * white_keys_found + offset
            w = key_width

            midi_note = octave_n * 12 + key.note
            color = SURFACE_0 if midi_note in playing else TEXT

            pygame.draw.rect(screen, SURFACE_2, pygame.Rect(
                left, top, w + LINE_WIDTH, w_height))
            rect = pygame.Rect(
                left + LINE_WIDTH, top + LINE_WIDTH, w - LINE_WIDTH, w_height - LINE_WIDTH * 2)
            rect.top = top + LINE_WIDTH
            pygame.draw.rect(screen, color, rect)
            white_keys_found += 1

    # for (i, key) in enumerate(KEYS):
    for (i, key) in enumerate(KEYS):
        if not key.white_key:

            left = b_width * i + offset + LINE_WIDTH / 2

            midi_note = octave_n * 12 + key.note
            color = SURFACE_0 if midi_note in playing else CRUST

            rect = pygame.Rect(
                left, top, b_width + LINE_WIDTH, b_height + LINE_WIDTH)
            # rect.centerx = left
            pygame.draw.rect(screen, SURFACE_2, rect)

            rect = pygame.Rect(
                left + LINE_WIDTH, top + LINE_WIDTH, b_width - LINE_WIDTH, b_height - LINE_WIDTH)
            rect.top = top + LINE_WIDTH
            # rect.centerx = left
            pygame.draw.rect(screen, color, rect)
            black_keys_found += 1


def draw_piano(pygame, screen, state: StepperSynthState, top, playing):
    octave_w = (SCREEN_WIDTH - LINE_WIDTH * 3) / 10

    for i in range(10):
        draw_octave(pygame, screen, state, top + LINE_WIDTH, octave_w,
                    octave_w * i + LINE_WIDTH, i, playing)
        # break


def draw_step(pygame, screen, fonts, top: float, bottom: float, width: float, step_n: int, cursor: int):
    half_width = width / 2
    x = LINE_WIDTH * 4 + half_width + width * step_n
    y = (bottom - top) / 2 + top
    # print(f"step {step_n} centered at point, ({x}, {y})")

    w, h = (width - LINE_WIDTH * 4, width - LINE_WIDTH * 4)

    if step_n != cursor:
        w /= 2
        h /= 2

    rect = pygame.Rect(
        0, 0, w, h)
    rect.center = (x, y)
    pygame.draw.rect(screen, GREEN, rect)

    rect = pygame.Rect(
        0, 0, w - LINE_WIDTH * 2, h - LINE_WIDTH * 2)
    rect.center = (x, y)
    pygame.draw.rect(screen, BACKGROUND_COLOR, rect)

    text = f"{step_n}"
    font = fonts[3]

    if step_n == cursor:
        font = fonts[0]

    display = font.render(
        text, True, TEXT_COLOR_1)
    text_rect = display.get_rect()
    text_rect.center = (x + text_rect.width * 0.06, y)

    screen.blit(display, text_rect)


def draw_steps(pygame, screen, fonts, state: StepperSynthState, bottom, top, sequence):
    # pass
    # notes = enumerate(playing)
    width = (SCREEN_WIDTH - LINE_WIDTH * 8) / 16
    # draw_step(pygame, screen, state, top, width, 0, state.cursor)
    # print(len(sequence))

    for i in range(state.cursor, len(sequence) + state.cursor):
        draw_step(pygame, screen, fonts, top, bottom, width, i, state.cursor)


def mk_text(font, text, color=TEXT_COLOR_1):
    display = font.render(text, True, color)
    text_rect = display.get_rect()
    return (display, text_rect)


def do_draw_label(pygame, screen, fonts, top: float, bottom: float, l: float, r: float, label: str, value: str):
    x = (r - l) / 2 + l
    y = (bottom - top) / 2 + top
    font = fonts[2]

    # text = "Sequence"
    display, text_rect = mk_text(font, label)
    text_rect.centerx = x + text_rect.width * 0.06
    text_rect.bottom = y - LINE_WIDTH

    screen.blit(display, text_rect)

    display, text_rect = mk_text(font, value, color=TEXT_COLOR_2)
    text_rect.centerx = x + text_rect.width * 0.06
    text_rect.top = y + LINE_WIDTH
    screen.blit(display, text_rect)


def draw_labels(pygame, screen, fonts, state: StepperSynthState, bottom: float, top: float):
    thirds = [0] + [SCREEN_WIDTH * (i / 3) for i in range(1, 4)]
    l_r = [thirds[i:i+2] for i in range(0, len(thirds) - 1)]
    # chunk = ints[i:i+chunk_size]

    l, r = l_r[0]
    # draw_name(pygame, screen, fonts, top, bottom, l, r, state.name)
    do_draw_label(pygame, screen, fonts, top, bottom,
                  l, r, "Sequence", state.name)
    l, r = l_r[1]
    # draw_tempo(pygame, screen, fonts, top, bottom, l, r, state.tempo)
    do_draw_label(pygame, screen, fonts, top,
                  bottom, l, r, "Tempo", f"{state.tempo}")
    l, r = l_r[2]
    # draw_step_total(pygame, screen, fonts, top,
    #                 bottom, l, r, len(state.sequence.steps))
    do_draw_label(pygame, screen, fonts, top,
                  bottom, l, r, "Steps", f"{len(state.sequence.steps)}")


def draw_button(pygame, screen, font, l: float, r: float, top: float, height: float, label: str, selected: bool, text_color=[TEXT_COLOR_1, GREEN], border_color=[GREEN, TEXT_COLOR_2]):
    # draw_button(pygame, screen, font, l, r, top, button_h)
    border_color = border_color[0] if not selected else border_color[1]
    text_color = text_color[0] if not selected else text_color[1]
    w = r - l - LINE_WIDTH * 2
    h = height - LINE_WIDTH * 2
    center = (((r - l) / 2) + l, top + height / 2)

    rect = pygame.Rect(
        0, 0, w, h)
    # rect.center = (x, y)
    # rect.right = r + LINE_WIDTH
    # rect.top = top + LINE_WIDTH
    rect.center = center
    pygame.draw.rect(screen, border_color, rect)

    rect = pygame.Rect(
        0, 0, w - LINE_WIDTH * 2, h - LINE_WIDTH * 2)
    # rect.center = (x, y)
    # rect.right = r - LINE_WIDTH
    # rect.top = top + LINE_WIDTH
    rect.center = center
    pygame.draw.rect(screen, BACKGROUND_COLOR, rect)

    display, text_rect = mk_text(font, label, color=text_color)
    c = (center[0] + text_rect.width * 0.06,
         center[1] - text_rect.height * 0.06)
    text_rect.center = c
    # text_rect.bottom = y - LINE_WIDTH
    # text_rect.centery =

    screen.blit(display, text_rect)


def draw_buttons(pygame, screen, fonts, state: StepperSynthState, bottom: float, top: float):
    middle_section = SCREEN_WIDTH * (3 / 5)
    fifth = SCREEN_WIDTH * (1 / 5)
    sections = [fifth] + [middle_section *
                          (i / 3) + fifth for i in range(1, 4)]
    l_r = [sections[i:i+2] for i in range(0, len(sections) - 1)]
    h = bottom - top
    button_h = h * (3.0 / 8.0)
    middle_y = h / 2 + top
    font = fonts[0]

    l, r = l_r[0]
    # draw last step button
    draw_button(pygame, screen, font, l, r, middle_y -
                button_h * 0.5, button_h, "<<<", False)
    l, r = l_r[1]
    # Play button
    draw_button(pygame, screen, font, l, r, middle_y -
                button_h - button_h * (1.0 / 8.0), button_h, "PLAY", state.playing, text_color=[TEXT_COLOR_2, GREEN], border_color=[TEXT_COLOR_2, GREEN])
    # rec button
    draw_button(pygame, screen, font, l, r, middle_y +
                button_h * (1.0 / 8.0), button_h, "RECORD", state.recording, text_color=[TEXT_COLOR_2, RED], border_color=[TEXT_COLOR_2, RED])
    l, r = l_r[2]
    # draw next step button
    draw_button(pygame, screen, font, l, r, middle_y -
                button_h * 0.5, button_h, ">>>", False)


def draw_stepper(pygame, screen, fonts, state: StepperSynthState):
    third = SCREEN_HEIGHT / 3
    bottom_h = third * 2
    bottom_row_h = bottom_h / 3

    playing = state.step.on_enter  # if state.playing else []
    # print("first_playing =", playing)
    playing = [step.note for (_, step) in playing if isinstance(
        step, StepCmd.Play)]
    # print("second_playing =", playing)

    draw_piano(pygame, screen, state, SCREEN_HEIGHT - bottom_row_h, playing)
    draw_steps(pygame, screen, fonts, state, SCREEN_HEIGHT -
               bottom_row_h, SCREEN_HEIGHT - bottom_row_h * 2, state.sequence.steps)
    draw_labels(pygame, screen, fonts, state, SCREEN_HEIGHT -
                bottom_row_h * 2, SCREEN_HEIGHT - bottom_row_h * 4)
    draw_buttons(pygame, screen, fonts, state, bottom_row_h, 0.0)


def stepper_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    if not select_mod_pressed(controller):
        return

    up = buttons.get("up")
    down = buttons.get("down")
    left = buttons.get("left")
    right = buttons.get("right")

    if controller.just_released(up) and not state.playing:
        synth.start_playing()
    elif controller.just_released(down) and not state.recording:
        synth.start_recording()
    elif (controller.just_released(up) or controller.just_released(down)) and (state.playing or state.recording):
        synth.stop_seq()
    elif controller.just_released(left):
        synth.prev_sequence()
        # synth.set_screen(Screen.Stepper(state.seq_n - 1))
    elif controller.just_released(right):
        synth.next_sequence()
        # synth.set_screen(Screen.Stepper(state.seq_n + 1))

    return synth
