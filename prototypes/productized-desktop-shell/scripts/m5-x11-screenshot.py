#!/usr/bin/env python3
"""Capture the visible desktop or Syn workbench window via Gdk."""

import os
import sys
from pathlib import Path

os.environ["GDK_BACKEND"] = "x11"
os.environ.setdefault("DISPLAY", os.environ.get("DISPLAY") or ":0")
os.environ.pop("WAYLAND_DISPLAY", None)

import gi

gi.require_version("Gdk", "3.0")
gi.require_version("GdkX11", "3.0")
from gi.repository import Gdk, GdkX11  # noqa: E402
import ctypes
import ctypes.util


def window_title(window) -> str:
    try:
        return window.get_title() or ""
    except Exception:
        return ""


def pick_window():
    display = Gdk.Display.get_default()
    if display is None:
        raise RuntimeError("gdk_display_unavailable")
    screen = display.get_default_screen()
    preferred = None
    largest = None
    largest_area = 0
    for window in screen.get_window_stack() or []:
        if window.get_state() & Gdk.WindowState.WITHDRAWN:
            continue
        geometry = window.get_geometry()
        width = geometry.width if hasattr(geometry, "width") else geometry[2]
        height = geometry.height if hasattr(geometry, "height") else geometry[3]
        title = window_title(window)
        if width >= 400 and height >= 300 and ("Codex" in title or "治理" in title or "工作台" in title):
            preferred = window
            break
        area = max(width, 0) * max(height, 0)
        if area > largest_area and width >= 400 and height >= 300:
            largest = window
            largest_area = area
    if preferred or largest:
        return preferred or largest
    foreign = largest_mapped_x11_window(display)
    return foreign or screen.get_root_window()


def largest_mapped_x11_window(display):
    x11_name = ctypes.util.find_library("X11")
    if not x11_name:
        return None
    x11 = ctypes.CDLL(x11_name)
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    raw = x11.XOpenDisplay(None)
    if not raw:
        return None

    class XWindowAttributes(ctypes.Structure):
        _fields_ = [
            ("x", ctypes.c_int),
            ("y", ctypes.c_int),
            ("width", ctypes.c_int),
            ("height", ctypes.c_int),
            ("border_width", ctypes.c_int),
            ("depth", ctypes.c_int),
            ("visual", ctypes.c_void_p),
            ("root", ctypes.c_ulong),
            ("klass", ctypes.c_int),
            ("bit_gravity", ctypes.c_int),
            ("win_gravity", ctypes.c_int),
            ("backing_store", ctypes.c_int),
            ("backing_planes", ctypes.c_ulong),
            ("backing_pixel", ctypes.c_ulong),
            ("save_under", ctypes.c_int),
            ("colormap", ctypes.c_ulong),
            ("map_installed", ctypes.c_int),
            ("map_state", ctypes.c_int),
            ("all_event_masks", ctypes.c_long),
            ("your_event_mask", ctypes.c_long),
            ("do_not_propagate_mask", ctypes.c_long),
            ("override_redirect", ctypes.c_int),
            ("screen", ctypes.c_void_p),
        ]

    x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    x11.XDefaultRootWindow.restype = ctypes.c_ulong
    x11.XQueryTree.argtypes = [
        ctypes.c_void_p,
        ctypes.c_ulong,
        ctypes.POINTER(ctypes.c_ulong),
        ctypes.POINTER(ctypes.c_ulong),
        ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)),
        ctypes.POINTER(ctypes.c_uint),
    ]
    x11.XGetWindowAttributes.argtypes = [
        ctypes.c_void_p,
        ctypes.c_ulong,
        ctypes.POINTER(XWindowAttributes),
    ]
    x11.XFetchName.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(ctypes.c_char_p)]
    best = None
    best_area = 0

    def walk(xid, depth=0):
        nonlocal best, best_area
        attrs = XWindowAttributes()
        if x11.XGetWindowAttributes(raw, xid, ctypes.byref(attrs)) == 0:
            return
        mapped = attrs.map_state == 2
        area = attrs.width * attrs.height
        if mapped and attrs.width >= 640 and attrs.height >= 480 and area > best_area:
            name = ctypes.c_char_p()
            x11.XFetchName(raw, xid, ctypes.byref(name))
            title = name.value.decode("utf-8", "replace") if name.value else ""
            best = (xid, title, attrs.width, attrs.height)
            best_area = area
        if depth >= 3:
            return
        root_ret = ctypes.c_ulong()
        parent = ctypes.c_ulong()
        children = ctypes.POINTER(ctypes.c_ulong)()
        count = ctypes.c_uint()
        if x11.XQueryTree(
            raw,
            xid,
            ctypes.byref(root_ret),
            ctypes.byref(parent),
            ctypes.byref(children),
            ctypes.byref(count),
        ) == 0:
            return
        for index in range(count.value):
            walk(children[index], depth + 1)

    walk(x11.XDefaultRootWindow(raw))
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    x11.XCloseDisplay(raw)
    if not best:
        sys.stderr.write("x11_no_mapped_app_window\n")
        return None
    sys.stderr.write(f"x11_selected_window xid={best[0]} {best[2]}x{best[3]} title={best[1]!r}\n")
    return GdkX11.X11Window.foreign_new_for_display(display, best[0])


def pixbuf_to_ppm(pixbuf, out: Path) -> None:
    width = pixbuf.get_width()
    height = pixbuf.get_height()
    n_channels = pixbuf.get_n_channels()
    rowstride = pixbuf.get_rowstride()
    pixels = pixbuf.get_pixels()
    rows = []
    for y in range(height):
        start = y * rowstride
        line = pixels[start : start + width * n_channels]
        rgb = bytearray()
        for x in range(width):
            offset = x * n_channels
            rgb.extend(line[offset : offset + 3])
        rows.append(bytes(rgb))
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("wb") as handle:
        handle.write(f"P6\n{width} {height}\n255\n".encode("ascii"))
        handle.write(b"".join(rows))


def main() -> int:
    if len(sys.argv) != 2:
        sys.stderr.write("usage: m5-x11-screenshot.py OUT.ppm\n")
        return 64
    out = Path(sys.argv[1])
    window = pick_window()
    geometry = window.get_geometry()
    # Gdk 3 get_geometry returns (x, y, width, height) or a Gdk.Rectangle.
    if hasattr(geometry, "width"):
        width, height = geometry.width, geometry.height
    else:
        width, height = geometry[2], geometry[3]
    if width <= 0 or height <= 0:
        sys.stderr.write("gdk_geometry_invalid\n")
        return 5
    pixbuf = Gdk.pixbuf_get_from_window(window, 0, 0, width, height)
    if pixbuf is None:
        root = Gdk.Display.get_default().get_default_screen().get_root_window()
        root_geometry = root.get_geometry()
        if hasattr(root_geometry, "width"):
            width, height = root_geometry.width, root_geometry.height
        else:
            width, height = root_geometry[2], root_geometry[3]
        pixbuf = Gdk.pixbuf_get_from_window(root, 0, 0, width, height)
    if pixbuf is None:
        sys.stderr.write("gdk_pixbuf_capture_failed\n")
        return 6
    pixbuf_to_ppm(pixbuf, out)
    sys.stdout.write(f"{out}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
