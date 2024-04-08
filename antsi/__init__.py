"""
A quick and user-friendly way to style your text using ANSI codes.

Built around a delightfully simple markup language, there's no messing about with codes or style resets. All the ANSI
code conversion, handling of overlapping styles, and terminal support is automatically handled for you.
"""

from ._antsi import ColorizeError, colorize, escape

__all__ = ["ColorizeError", "colorize", "escape"]
