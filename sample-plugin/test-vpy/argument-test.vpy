import vapoursynth as vs
from vapoursynth import core

try:
    running_from_test
except NameError:
    core.std.LoadPlugin('../../target/debug/libsample_plugin.so')

def make_frame(width):
    return core.std.BlankClip(width = width,
                              height = 240,
                              format = vs.GRAY8,
                              length = 1,
                              color = [0])

core.vapoursynth_rs.ArgumentTest(
    42,
    1337.0,
    "asd",
    make_frame(320),
    make_frame(320).get_frame(0),
    lambda: 10,
    optional_int = 123,
    frame_array = [make_frame(256).get_frame(0), make_frame(64).get_frame(0)]).set_output()
