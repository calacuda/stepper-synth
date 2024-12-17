def set_max(value: float, max: float):
    res = value if value <= max else max
    return res if res > 0.0 else 0.0
