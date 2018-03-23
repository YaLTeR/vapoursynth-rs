import vapoursynth as vs
from vapoursynth import core

def make_clip(format, color):
    return core.std.BlankClip(width = 1920,
                              height = 1080,
                              format = format,
                              length = 100,
                              fpsnum = 60,
                              fpsden = 1,
                              color = color)

make_clip(vs.YUV420P10, [789, 123, 456]).set_output(0)
make_clip(vs.YUV444PS, [5.0, 42.0, 0.25]).set_output(1)
make_clip(vs.RGBS, [0.125, 10.0, 0.5]).set_output(2)
format = core.get_format(vs.YUV444P8).replace(bits_per_sample = 17)
make_clip(format, [77777, 88888, 99999]).set_output(3)
format = core.get_format(vs.YUV444P8).replace(bits_per_sample = 32)
make_clip(format, [2**32-1, 12345, 65432]).set_output(4)
