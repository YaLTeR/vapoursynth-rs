import vapoursynth as vs
from vapoursynth import core
core.std.LoadPlugin('../../target/debug/libsample_filter.so')

clip = core.vapoursynth_rs.RandomNoise(width = 320,
                                       height = 240,
                                       format = vs.RGB24,
				       length = 10,
                                       fpsnum = 60,
                                       fpsden = 1)
clip = core.resize.Lanczos(clip, format = vs.YUV444P8, matrix_s = "709")
clip.set_output()
