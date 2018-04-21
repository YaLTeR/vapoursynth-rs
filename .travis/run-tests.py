#!/usr/bin/python3
import itertools, os, subprocess, sys

if __name__ == '__main__':
    VS_API_VERSIONS = ["vapoursynth-api-" + str(v) for v in range(31, 36)]
    VSSCRIPT_API_VERSIONS = ["vsscript-api-" + str(v) for v in range(31, 33)]
    VAPOURSYNTH_FUNCTIONS = ["vapoursynth-functions"]
    VSSCRIPT_FUNCTIONS = ["vsscript-functions"]
    F16_PIXEL_TYPE = ["f16-pixel-type"]

    features = [
        VS_API_VERSIONS, VSSCRIPT_API_VERSIONS, VAPOURSYNTH_FUNCTIONS,
        VSSCRIPT_FUNCTIONS, F16_PIXEL_TYPE
    ]

    for f in features:
        f += [""]

    someone_failed = False

    for features in itertools.product(*features):
        features_string = str.join(' ', features)
        print("Starting tests with features: " + features_string)
        sys.stdout.flush()

        returncode = subprocess.call(
            ['cargo', 'test', '--verbose', '--features', features_string])
        if returncode != 0:
            someone_failed = True
            print("TEST FAILURE: " + features_string)

    if someone_failed:
        print("One of the tests failed, exiting with code 1.")
        sys.exit(1)
