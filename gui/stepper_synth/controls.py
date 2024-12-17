from dataclasses import dataclass
from copy import deepcopy


buttons = {
    "a": 1,
    "b": 0,
    "x": 3,
    "y": 2,
    "start": 7,
    "select": 6,
    "lb": 4,
    "rb": 5,
    "lt": 9,
    "rt": 10,
    "home": 8,
    "up": (0, 1),
    "down": (0, -1),
    "left": (-1, 0),
    "right": (1, 0)
}


@dataclass
class ButtonData:
    pressed: bool
    just_pressed: bool
    just_released: bool
    pressed_time: int

    def step(self):
        if self.just_pressed:
            self.just_pressed = False

        if self.just_released:
            self.just_released = False

    def press(self):
        self.pressed = True
        self.just_pressed = True
        # TODO: set pressed time

    def release(self):
        self.pressed = False
        self.just_released = True


class Buttons:
    def __init__(self) -> None:
        # self.buttons = {}
        self.pressed_now = []
        self.last_pressed_now = []

    def press(self, button):
        if button not in self.pressed_now:
            # print(f"pressed {button}, {self.pressed_now}")
            self.pressed_now.append(button)
        else:
            self.release(button)

    def release(self, button):
        self.pressed_now = [
            but for but in self.pressed_now if but is not button]

    def purge_dpad(self):
        self.pressed_now = [
            but for but in self.pressed_now if type(but) is not tuple]

    def is_pressed(self, button) -> bool:
        return button in self.pressed_now

    def just_pressed(self, button) -> bool:
        return self.is_pressed(button) and button not in self.last_pressed_now

    def just_released(self, button) -> bool:
        return button in self.last_pressed_now and not self.is_pressed(button)

    def step(self):
        self.last_pressed_now = deepcopy(self.pressed_now)
