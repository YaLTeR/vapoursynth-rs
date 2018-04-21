import vapoursynth as vs
from vapoursynth import core

def make_clip(format, color):
    return core.std.BlankClip(width = 320,
                              height = 240,
                              format = format,
                              length = 100,
                              fpsnum = 60,
                              fpsden = 1,
                              color = color)

make_clip(vs.YUV420P10, [789, 123, 456]).set_output(0)
make_clip(vs.YUV444PS, [5.0, 42.0, 0.25]).set_output(1)
make_clip(vs.RGBS, [0.125, 10.0, 0.5]).set_output(2)
format = core.register_format(vs.YUV, vs.INTEGER, 17, 0, 0)
make_clip(format.id, [77777, 88888, 99999]).set_output(3)
format = core.register_format(vs.YUV, vs.INTEGER, 32, 0, 0)
make_clip(format.id, [2**32-1, 12345, 65432]).set_output(4)
make_clip(vs.RGBH, [0.0625, 5.0, 0.25]).set_output(5)
