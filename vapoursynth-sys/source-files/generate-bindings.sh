#!/bin/sh

bindgen --whitelist-function 'getVapourSynthAPI' \
        --whitelist-function 'vsscript.*' \
        --whitelist-type 'VSColorFamily' \
        --whitelist-type 'VSSampleType' \
        --whitelist-type 'VSPresetFormat' \
        --whitelist-type 'VSFilterMode' \
        --whitelist-type 'VSNodeFlags' \
        --whitelist-type 'VSPropTypes' \
        --whitelist-type 'VSGetPropErrors' \
        --whitelist-type 'VSPropAppendMode' \
        --whitelist-type 'VSActivationReason' \
        --whitelist-type 'VSMessageType' \
        --whitelist-type 'VSEvalFlags' \
        --whitelist-type 'VSInitPlugin' \
        --blacklist-type '__int64_t' \
        --blacklist-type '__uint8_t' \
        --bitfield-enum 'VSNodeFlags' \
        --rustified-enum 'VS[^N].*' \
        --no-layout-tests \
        -o bindings.rs \
        vapoursynth/include/VSScript.h \
        -- -target x86_64-unknown-windows-unknown
