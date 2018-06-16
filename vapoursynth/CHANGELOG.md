## v0.2 (16th Jun 2018)
- Added plugin support! That includes:
  - `plugins::{Metadata,Filter,FilterFunction}` types and traits for making plugins;
  - `export_vapoursynth_plugin!` macro for exporting a VapourSynth plugin;
  - `make_filter_function!` macro for making filters without much boilerplate.
- Added a sample plugin in the `sample-filter` folder.
- Added the `component::Component` trait and `Frame::plane*()` accessors for safely working with the pixel data without having to manually transmute slices, including an optional half-precision float support using the `half` crate.
- Added `plugin::Plugin` and other relevant APIs for enumerating plugins and invoking their functions.
- Added lifetime parameters to many types to fix soundness issues.
- Split `Frame` into `Frame`, `FrameRef`, `FrameRefMut`.
- Added the `map::Value` trait and generic `Map::{get,get_iter,set,append}()` functions.
- Added format caching in `Frame` to reduce the number of API calls needed.
- Added some convenience `From` impls.

### v0.1.2 (2nd Apr 2018)
- Fixed `Frame::data_row()` returning slices of incorrect rows (using the `plane` value instead of the `row` value).

### v0.1.1 (24th Mar 2018)
- Added support for targetting 32-bit Windows
- Added automatic detection of common Windows VapourSynth library dirs
- Fixed `Frame::data()` and `Frame::data_row()` returning slices of incorrect sizes (too short) for pixel formats with more than 1 byte per pixel

## v0.1.0
- Initial release
