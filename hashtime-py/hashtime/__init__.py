from ._hashtime import (
    generate,
    generate_with_callback,
    compare,
    restore_times,
    FileHashTimeResult,
    FileTimeResult,
    Diff,
    FieldDiff,
)

__doc__ = _hashtime.__doc__
__all__ = [
    "generate",
    "generate_with_callback",
    "compare",
    "restore_times",
    "FileHashTimeResult",
    "FileTimeResult",
    "Diff",
    "FieldDiff",
]
