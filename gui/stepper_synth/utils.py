from typing import Callable, Tuple


def set_max(value: float, max: float, min: float = 0.0):
    res = value if value <= max else max
    return res if res > min else min


# def draw_text(screen, font, text, color,)

def do_nothing(rect):
    return rect


def draw_text(screen, text: str, font, where: Tuple[float, float], color, rect_callback=do_nothing):
    display = font.render(
        text, True, color)
    text_rect = display.get_rect()
    x, y = where
    text_rect.center = (int(x), int(y))
    text_rect = rect_callback(text_rect)

    screen.blit(display, text_rect)


def draw_diagonal_text(screen, text: str, font, where: Tuple[float, float], color, angle=-45, rect_callback=do_nothing):
    import pygame

    display = font.render(
        text, True, color)
    display = pygame.transform.rotate(display, angle)
    text_rect = display.get_rect()
    x, y = where
    text_rect.center = (int(x), int(y))
    text_rect = rect_callback(text_rect)

    screen.blit(display, text_rect)


def timer_is_done(pygame, timer, seconds_since=0.1) -> bool:
    return (pygame.time.get_ticks() - timer) / 1000 >= seconds_since
