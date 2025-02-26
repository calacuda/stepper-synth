from stepper_synth_backend import StepperSynthState, StepperSynth, WTSynthParam
from .controls import Buttons, buttons
from .config import *
from .utils import *
from .full_dial import draw_dial


FIRST_I = 0
COL_I = 0
Y_I = 0
SRC_S = [
    "Velocity",
    "Env-0",
    "Env-1",
    "Env-2",
    "Env-3",
    "Lfo-0",
    "Lfo-1",
    "Lfo-2",
    "Lfo-3",
    "Gate",
    "Macro1",
    "Macro2",
    "Macro3",
    "Macro4",
    "ModWheel",
    "PitchWheel",
]


class DummyIndex:
    def __init__(self) -> None:
        pass

    def __getitem__(self, _i):
        return None

    def __len__(self):
        return 1


class DestIndex:
    # dest = "[Osc]\nosc = 1\nparam = \"Level\""
    tokens = [
        ["[Osc]", "[Env]", "[Lfo]", "[LowPass]", "[SynthVolume]"],
        [("osc = 0", "osc = 1", "osc = 2"), ("env = 0", "env = 1", "env = 2", "env = 3"),
         ("lfo = 0", "lfo = 1", "lfo = 2", "lfo = 3"), ("low_pass = \"LP1\"", "low_pass = \"LP2\""), DummyIndex()],
        [("param = \"Level\"", "param = \"Tune\""),
         ("param = \"Atk\"", "param = \"Dcy\"",
          "param = \"Sus\"", "param = \"Rel\""),
         ("param = \"Speed\"",),
         ("param = \"Cutoff\"", "param = \"Res\"", "param = \"Mix\""), DummyIndex()],
    ]
    # lens = [len(tokens[0]), len(tokens[1]), len(tokens[2])]
    display_tokens = [
        ["Osc", "Env", "Lfo", "LowPass", "Volume"],
        [("1", "2", "3"), ("1", "2", "3", "4"),
         ("1", "2", "3", "4"), ("1", "2"), DummyIndex()],
        [("Level", "Tune"), ("Atk", "Dcy", "Sus", "Rel"), ("Speed",),
         ("Cutoff", "Res", "Mix"), DummyIndex()],
    ]

    def __init__(self) -> None:
        self.i_s = [0, 0, -1]
        self.meta_i = 2

    def gen(self, tokens, sep: str) -> str:
        main_i = self.i_s[0]

        loc_toks = [tokens[0][main_i], tokens[1][main_i]
                    [self.i_s[1]], tokens[2][main_i][self.i_s[2]]]
        toks = [tok for tok in loc_toks if tok is not None]

        return sep.join(toks)

    def gen_display(self) -> str:
        return self.gen(self.display_tokens, " ")

    def gen_cmd(self) -> str:
        return self.gen(self.tokens, "\n")

    def _inc(self, meta_i):
        # self.i_s[self.meta_i] == len(self.tokens[self.meta_i][self.i_s[0]])
        if meta_i < 0:
            self.i_s = [0, 0, -1]
            self.meta_i = 2

        self.i_s[meta_i] += 1

        if meta_i == 0:
            self.i_s[meta_i] %= len(self.tokens[0])

        # print(f"before {self.i_s} | {
        #       meta_i} -> {self.tokens}[{meta_i}][{self.i_s[0]}]")
        if self.i_s[meta_i] == len(self.tokens[meta_i][self.i_s[0]]):
            self.i_s[meta_i] = 0
            self._inc(meta_i - 1)

    def inc(self):
        self._inc(self.meta_i)

    def _dec(self, meta_i):
        # self.i_s[self.meta_i] == len(self.tokens[self.meta_i][self.i_s[0]])
        if meta_i < 0:
            self.i_s = [0, 0, -1]
            self.meta_i = 2
            return

        # print("meta_i", meta_i)

        self.i_s[meta_i] -= 1

        # if meta_i == 0:
        # self.i_s[meta_i] %= len(self.tokens[meta_i][self.i_s[0]])

        # print(f"before {self.i_s} | {
        #       meta_i} -> {self.tokens}[{meta_i}][{self.i_s[0]}]")
        if self.i_s[meta_i] < 0:
            self._dec(meta_i - 1)
            self.i_s[meta_i] = len(self.tokens[meta_i][self.i_s[0]]) - 1

    def dec(self):
        # print("decrease")
        self._dec(self.meta_i)
        # print(f"{self.i_s}")


class NewRow:
    def __init__(self) -> None:
        self.src = ""
        self.dest = ""
        self.bipolar = False
        self.amt = 0.5
        self.display = False
        self.matrix_size = 0
        self.waiting_for_add = False
        self._reset()

    def _reset(self):
        self.src = "--"
        self.dest = "--"
        self.bipolar = False
        self.amt = 0.5
        self.display = False
        self.waiting_for_add = False
        self.src_i = -1
        self.dest_i = DestIndex()

    def toggle_display(self):
        if not self.display:
            self._reset()

        self.display = not self.display

    def check_was_added(self, state: StepperSynthState):
        # and self.waiting_for_add:
        if len(state.mod_matrix) > self.matrix_size:
            self._reset()
            self.matrix_size = len(state.mod_matrix)

    def add(self, synth: StepperSynth, state: StepperSynthState):
        self.matrix_size = len(state.mod_matrix)
        # self.waiting_for_add = True
        synth.wt_param_setter(WTSynthParam.ModMatrixAdd(
            self.src, self.dest_i.gen_cmd(), self.amt, self.bipolar))

        return synth

    def nudge_src(self, forward: bool):
        if forward:
            self.src_i += 1
        else:
            self.src_i -= 1

        self.src_i %= len(SRC_S)
        self.src = SRC_S[self.src_i]

    def nudge_dest(self, forward: bool):
        if forward:
            self.dest_i.inc()
        else:
            self.dest_i.dec()

        self.dest = self.dest_i.gen_display()


NEW_ROW = NewRow()
MOVE_TIMER = 0
ADJUST_TIMER = 0


def move_cursor(pygame, controller: Buttons, state: StepperSynthState):
    global COL_I
    global FIRST_I
    global Y_I
    global MOVE_TIMER

    # print(f"({X_I}, {Y_I})")

    if select_mod_pressed(controller) or not timer_is_done(pygame, MOVE_TIMER, 0.15):
        return

    matrix_len = len(state.mod_matrix) - FIRST_I

    if NEW_ROW.display:
        matrix_len += 1

    n_col = min((matrix_len, 8))

    if controller.is_pressed(buttons.get("up")):
        Y_I -= 1
        Y_I %= 4
    elif controller.is_pressed(buttons.get("down")):
        Y_I += 1
        Y_I %= 4
    elif controller.is_pressed(buttons.get("right")):
        if COL_I == 7 and len(state.mod_matrix) - FIRST_I >= 7:
            FIRST_I += 1
        else:
            COL_I += 1
            COL_I %= n_col

    elif controller.is_pressed(buttons.get("left")):
        if COL_I == 0 and FIRST_I > 0:
            FIRST_I -= 1
        else:
            COL_I -= 1
            COL_I %= n_col
    else:
        return

    MOVE_TIMER = pygame.time.get_ticks()


def toggle_bipol(forward: bool):
    global NEW_ROW

    NEW_ROW.bipolar = not NEW_ROW.bipolar


def nudge_amt(forward: bool):
    global NEW_ROW

    amt = 0.005 if forward else -0.005

    NEW_ROW.amt += amt


def nudge_src(forward: bool):
    global NEW_ROW

    NEW_ROW.nudge_src(forward)


def nudge_dest(forward: bool):
    global NEW_ROW

    NEW_ROW.nudge_dest(forward)


def adjust_value(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState):
    global ADJUST_TIMER
    global NEW_ROW

    matrix = state.mod_matrix
    matrix_i = min((len(matrix) - FIRST_I, 7))

    if NEW_ROW.display:
        NEW_ROW.check_was_added(state)

    if controller.just_released(buttons.get("a")) and not NEW_ROW.display:
        NEW_ROW.display = True
    elif controller.just_released(buttons.get("a")) and NEW_ROW.display:
        synth = NEW_ROW.add(synth, state)

    if (not select_mod_pressed(controller)) or (not timer_is_done(pygame, ADJUST_TIMER)) or (not NEW_ROW.display) or (not COL_I == matrix_i):
        # TIMER = pygame.time.get_ticks()
        return synth

    # print(synth.)

    nudge = [
        # nudge src
        nudge_src,
        # nudge amt
        nudge_amt,
        # toggle bipolar
        toggle_bipol,
        # nudge dest
        nudge_dest,
    ]

    if controller.is_pressed(buttons.get("right")):
        nudge[Y_I](True)
    elif controller.is_pressed(buttons.get("left")):
        nudge[Y_I](False)
    else:
        return synth

    # TODO: add deleting and modding of mod matrix entries

    ADJUST_TIMER = pygame.time.get_ticks()

    return synth


def draw_src(screen, fonts, entry_src, top, bottom, x, sel):
    y = (top + bottom) / 2
    draw_diagonal_text(screen, entry_src,
                       fonts[1], (x, y), TEXT_COLOR_2 if not sel else RED)


def draw_matrix(pygame, screen, fonts, entry, top, left, row_width, i):
    right = left + row_width
    x = (right + left) / 2
    col_sel = i == COL_I

    indicator_color = GREEN if not col_sel else RED

    draw_text(screen, f"{i + FIRST_I + 1}",
              fonts[0], (x, row_width / 4), indicator_color)

    pygame.draw.line(screen, GREEN, (left, row_width / 2),
                     (right, row_width / 2), int(LINE_WIDTH / 2))
    top = top + (row_width / 2)
    h = SCREEN_HEIGHT - top
    bottom = top + (h * (7 / 16))
    draw_src(screen, fonts, entry.src, top, bottom, x, col_sel and Y_I == 0)
    top = bottom
    bottom = top + (h * (1 / 8))
    # draw amt and Bi/uni-polar
    y = (top + bottom) / 2
    text_x = (left * 3 + right) / 4
    sel = col_sel and Y_I == 1
    draw_diagonal_text(screen, f"{entry.amt:.3f}",
                       fonts[3], (text_x, y), TEXT_COLOR_1 if not sel else RED)
    text_x = (left + right * 3) / 4
    direction = "Bi-pol" if entry.bipolar else "Uni-pol"
    sel = col_sel and Y_I == 2
    draw_diagonal_text(
        screen, direction, fonts[3], (text_x, y), TEXT_COLOR_1 if not sel else RED)

    top = bottom
    bottom = top + (h * (7 / 16))
    # draw dest
    draw_src(screen, fonts, entry.dest, top, bottom, x, col_sel and Y_I == 3)


def draw_new_matrix(pygame, screen, fonts, top, left, row_width, i):
    right = left + row_width
    x = (right + left) / 2
    col_sel = i == COL_I

    indicator_color = GREEN if not col_sel else RED

    draw_text(screen, "*",
              fonts[0], (x, row_width / 4), indicator_color)

    pygame.draw.line(screen, GREEN, (left, row_width / 2),
                     (right, row_width / 2), int(LINE_WIDTH / 2))
    top = top + (row_width / 2)
    h = SCREEN_HEIGHT - top
    bottom = top + (h * (7 / 16))
    draw_src(screen, fonts, NEW_ROW.src, top, bottom, x, col_sel and Y_I == 0)
    top = bottom
    bottom = top + (h * (1 / 8))
    # draw amt and Bi/uni-polar
    y = (top + bottom) / 2
    text_x = (left * 3 + right) / 4
    sel = col_sel and Y_I == 1
    draw_diagonal_text(screen, f"{NEW_ROW.amt:.3f}",
                       fonts[3], (text_x, y), TEXT_COLOR_1 if not sel else RED)
    text_x = (left + right * 3) / 4
    direction = "Bi-pol" if NEW_ROW.bipolar else "Uni-pol"
    sel = col_sel and Y_I == 2
    draw_diagonal_text(
        screen, direction, fonts[3], (text_x, y), TEXT_COLOR_1 if not sel else RED)

    top = bottom
    bottom = top + (h * (7 / 16))
    # draw dest
    draw_src(screen, fonts, NEW_ROW.dest, top, bottom, x, col_sel and Y_I == 3)


def draw_mod_menu(pygame, screen, fonts, synth: StepperSynthState):
    matrix = synth.mod_matrix
    row_width = SCREEN_WIDTH / 8

    # pygame.draw.line(screen, GREEN, (0, row_width / 2),
    #                  (SCREEN_WIDTH, row_width / 2), int(LINE_WIDTH / 2))

    for i in range(1, 8):
        x = i * row_width
        # draw vertical divider
        pygame.draw.line(screen, GREEN, (x, 0),
                         (x, SCREEN_HEIGHT), int(LINE_WIDTH / 2))

    n = 7 if NEW_ROW.display else 8

    for i in range(n):
        top = 0
        left = i * row_width
        # print(matrix[i + FIRST_I])

        if i + FIRST_I >= len(matrix):
            break

        draw_matrix(pygame, screen, fonts,
                    matrix[i + FIRST_I], top, left, row_width, i)

    if NEW_ROW.display:
        top = 0
        i = min((len(matrix) - FIRST_I, 7))
        left = i * row_width
        draw_new_matrix(pygame, screen, fonts, top, left, row_width, i)


def mod_menu_controls(pygame, controller: Buttons, synth: StepperSynth, state: StepperSynthState) -> StepperSynth:
    move_cursor(pygame, controller, state)
    return adjust_value(pygame, controller, synth, state)
    # return synth
