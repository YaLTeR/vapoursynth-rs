import vapoursynth as vs
from vapoursynth import core

try:
    running_from_test
except NameError:
    core.std.LoadPlugin('../../target/debug/libsample_plugin.so')

clip = core.vapoursynth_rs.RandomNoise(width = 320,
                                       height = 240,
                                       format = vs.RGB24,
                                       length = 10,
                                       fpsnum = 60,
                                       fpsden = 1)

try:
    running_from_test
except NameError:
    clip = core.resize.Lanczos(clip, format = vs.YUV444P8, matrix_s = "709")

clip.set_output()
