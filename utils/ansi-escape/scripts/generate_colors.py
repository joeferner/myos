# print a list of the 256-color red/green/blue values used by xterm.
#
# reference:
# https://en.wikipedia.org/wiki/ANSI_escape_code
# https://github.com/ThomasDickey/ncurses-snapshots/blob/master/test/xterm-16color.dat
# https://github.com/ThomasDickey/xterm-snapshots/blob/master/XTerm-col.ad
# https://github.com/ThomasDickey/xterm-snapshots/blob/master/256colres.pl

def print_color(code, r, g, b):
    print(f"// {code}\nColor {{ red:{r}, green: {g}, blue: {b} }},")

# colors 0-16 correspond to the ANSI and aixterm naming
for code in range(0, 16):
    if code > 8:
        level = 255
    elif code == 7:
        level = 229
    else:
        level = 205
    r = 127 if code == 8 else level if (code & 1) != 0 else 92 if code == 12 else 0
    g = 127 if code == 8 else level if (code & 2) != 0 else 92 if code == 12 else 0
    b = 127 if code == 8 else 238 if code == 4 else level if (code & 4) != 0 else 0
    print_color(code, r, g, b)

# colors 16-231 are a 6x6x6 color cube
for red in range(0, 6):
    for green in range(0, 6):
        for blue in range(0, 6):
            code = 16 + (red * 36) + (green * 6) + blue
            r = red   * 40 + 55 if red   != 0 else 0
            g = green * 40 + 55 if green != 0 else 0
            b = blue  * 40 + 55 if blue  != 0 else 0
            print_color(code, r, g, b)

# colors 232-255 are a grayscale ramp, intentionally leaving out black and white
code = 232
for gray in range(0, 24):
    level = gray * 10 + 8
    code = 232 + gray
    print_color(code, r, g, b)
