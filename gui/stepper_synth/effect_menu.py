from .controls import Buttons, buttons
from stepper_synth_backend import State, GuiParam, Knob, PythonCmd, EffectType, TrackerIPC
from .config import *
from .utils import *


effects = [EffectType.Reverb, EffectType.Chorus]
INDEX = 0


def draw_effect_menu(pygame, screen, fonts, synth_state: State):
    # engine = synth_state.engine
    rad = LINE_WIDTH * 2

    outer = pygame.Rect(SCREEN_WIDTH / 4, SCREEN_HEIGHT /
                        8, SCREEN_WIDTH / 4, (SCREEN_HEIGHT /
                                              8) * 6)
    outer.right = SCREEN_WIDTH

    outer.centery = SCREEN_HEIGHT / 2

    pygame.draw.rect(screen, GREEN, outer,
                     border_top_left_radius=rad, border_bottom_left_radius=rad)

    rad -= LINE_WIDTH
    iner = pygame.Rect(SCREEN_WIDTH / 4, SCREEN_HEIGHT /
                       8, SCREEN_WIDTH / 4 - LINE_WIDTH, (SCREEN_HEIGHT /
                                                          8) * 6 - LINE_WIDTH)
    iner.right = SCREEN_WIDTH
    iner.centery = SCREEN_HEIGHT / 2

    pygame.draw.rect(screen, BACKGROUND_COLOR, iner,
                     border_top_left_radius=rad, border_bottom_left_radius=rad)

    width = SCREEN_WIDTH / 4
    height = (SCREEN_HEIGHT / 8) * 6 - LINE_WIDTH
    left = SCREEN_WIDTH - width * 0.9
    line_height = height / 4
    offset = SCREEN_HEIGHT / 8 + LINE_WIDTH + line_height

    for i in range(3):
        y = i * line_height + offset
        pygame.draw.line(screen, GREEN, (left, y),
                         (SCREEN_WIDTH, y), width=LINE_WIDTH)

    offset = SCREEN_HEIGHT / 8 + LINE_WIDTH + line_height / 2
    # x = SCREEN_WIDTH - wdith / 2

    for (i, engine) in enumerate(effects):
        y = i * line_height + offset
        # print(engine, synth_state.engine, engine == synth_state.engine)
        prefix = "> " if i == INDEX else ""
        text = fonts[0].render(f'{prefix}{engine}', True, TEXT_COLOR_1)
        text_rect = text.get_rect()
        text_rect.centery = y
        text_rect.right = SCREEN_WIDTH
        screen.blit(text, text_rect)


def effect_menu_controles(ipc: TrackerIPC, controls: Buttons, synth_state: State):
    global INDEX

    if controls.just_released(buttons.get("up")):
        INDEX -= 1
        INDEX %= len(effects)
    elif controls.just_released(buttons.get("down")):
        INDEX += 1
        INDEX %= len(effects)
    elif controls.just_released(buttons.get("a")):
        new_effect = effects[INDEX]
        ipc.send(PythonCmd.ChangeEffectType(new_effect))
        return True


def reset_effect_menu():
    global INDEX

    INDEX = 0
