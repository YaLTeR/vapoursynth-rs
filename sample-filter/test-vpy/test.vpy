import vapoursynth as vs
from vapoursynth import core
core.std.LoadPlugin('../../target/debug/libsample_filter.so')

def make_frame(format, color):
 return core.std.BlankClip(width = 320,
                           height = 240,
                           format = format,
                           length = 1,
                           color = color)

clip = core.std.Splice([make_frame(vs.RGB24, [2**6, 2**6, 0]),
                        make_frame(vs.RGB27, [2**7, 2**7, 0]),
                        make_frame(vs.RGB30, [2**8, 2**8, 0]),
                        make_frame(vs.RGB48, [2**14, 2**14, 0])],
                       mismatch = True)

clip = core.vapoursynth_rs.Invert(clip)
clip = core.resize.Lanczos(clip, format = vs.YUV444P8, matrix_s = "709")
clip.set_output()
