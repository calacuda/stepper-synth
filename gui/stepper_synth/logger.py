from logging import getLogger, basicConfig, DEBUG, INFO, WARNING, ERROR, CRITICAL, Formatter, StreamHandler
from stepper_synth_backend import log_debug, log_info, log_warn, log_error


class CustomFormatter(Formatter):

    grey = "\x1b[38;20m"
    blue = "\x1b[34;20m"
    green = "\x1b[32;20m"
    yellow = "\x1b[33;20m"
    red = "\x1b[31;20m"
    bold_red = "\x1b[31;1m"
    reset = "\x1b[0m"
    metadata_fmt = "[%(levelname)s | %(asctime)s | %(name)s | (%(filename)s:%(lineno)d)]:"
    # log_header = f"{metadata_fmt}:{reset}"
    fmt = f"{metadata_fmt}:{reset}"
    # fmt = f"{log_header:<78} %(message)s"

    FORMATS = {
        DEBUG: f"{blue}",
        INFO: f"{green}",
        WARNING: f"{yellow}",
        ERROR: f"{red}",
        CRITICAL: f"{bold_red}",
    }
    rust_loggers = {
        DEBUG: log_debug,
        INFO: log_info,
        WARNING: log_warn,
        ERROR: log_error,
        CRITICAL: log_error,
    }

    def format(self, record):
        color = self.FORMATS.get(record.levelno)
        fmt_header = Formatter(self.metadata_fmt)
        fmt_msg = Formatter("%(message)s")
        header = fmt_header.format(record)

        rust_fmt = Formatter(
            "(PYTHON-FRONTEND said | %(filename)s:%(lineno)d) => %(message)s")
        rust_msg = rust_fmt.format(record)
        rust_log = self.rust_loggers[record.levelno]
        rust_log(rust_msg)

        return f"{color}{header:<81}{self.reset} => {fmt_msg.format(record)}"


def get_logger(name: str, log_level):

    logger = getLogger(name)
    # basicConfig(level=log_level)

    # return logger
    ch = StreamHandler()
    # ch.setLevel(log_level)
    logger.setLevel(log_level)

    ch.setFormatter(CustomFormatter())

    logger.addHandler(ch)

    return logger
