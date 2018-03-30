import vapoursynth as vs
from vapoursynth import core
core.std.LoadPlugin('../target/debug/libsample_filter.so')
clip = core.std.BlankClip(width = 1920,
                          height = 1080,
                          format = vs.RGB24,
                          length = 100,
                          fpsnum = 60,
                          fpsden = 1,
                          color = [255, 255, 0])
clip = core.invert.SampleName(clip)
clip = core.resize.Lanczos(clip, format = vs.YUV444P8, matrix_s = "709")
clip.set_output()
