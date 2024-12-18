def set_max(value: float, max: float, min: float = 0.0):
    res = value if value <= max else max
    return res if res > min else min
