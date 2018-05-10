import vapoursynth as vs
from vapoursynth import core

try:
    running_from_test
except NameError:
    core.std.LoadPlugin('../../target/debug/libsample_plugin.so')

def make_frame(format, color):
    return core.std.BlankClip(width = 320,
                              height = 240,
                              format = format,
                              length = 1,
                              color = color)

core.vapoursynth_rs.ArgumentTest(
    42,
    1337.0,
    "asd",
    make_frame(vs.GRAY8, [0]),
    make_frame(vs.GRAY8, [0]).get_frame(0),
    lambda: 10).set_output()
