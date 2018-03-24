## v0.1.1
- Added support for targetting 32-bit Windows
- Added automatic detection of common Windows VapourSynth library dirs
- Fixed `Frame::data()` and `Frame::data_row()` returning slices of incorrect sizes (too short) for pixel formats with more than 1 byte per pixel

### v0.1.0
- Initial release
