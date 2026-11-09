"""Probe actual X11 font loading without opening a game window or changing X11."""
import ctypes
import ctypes.util
import os


def probe():
    if not os.environ.get('DISPLAY'):
        raise ValueError('DISPLAY is required for visible play; use --prepare-only or resume after display repair')
    library = ctypes.util.find_library('X11')
    if not library:
        raise ValueError('X11 runtime library is unavailable')
    x = ctypes.CDLL(library)
    x.XOpenDisplay.argtypes = [ctypes.c_char_p]; x.XOpenDisplay.restype = ctypes.c_void_p
    x.XLoadQueryFont.argtypes = [ctypes.c_void_p, ctypes.c_char_p]; x.XLoadQueryFont.restype = ctypes.c_void_p
    x.XFreeFont.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    x.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = x.XOpenDisplay(None)
    if not display:
        raise ValueError('Cannot open DISPLAY; prepared branches can be resumed after display repair')
    explicit = os.environ.get('BORG_PLAY_FONT') or os.environ.get('ANGBAND_X11_FONT_0') or os.environ.get('ANGBAND_X11_FONT')
    candidates = [explicit] if explicit else ['-adobe-courier-medium-r-normal--28-*-*-*-p-*-iso8859-1', '10x20', '9x18', 'fixed']
    try:
        for name in candidates:
            font = x.XLoadQueryFont(display, name.encode())
            if font:
                x.XFreeFont(display, font)
                os.environ['ANGBAND_X11_FONT_0'] = name
                print('Loaded X11 main font:', name, flush=True)
                return name
    finally:
        x.XCloseDisplay(display)
    raise ValueError('X11 could not load requested font: ' + str(explicit or candidates))
