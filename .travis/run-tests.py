#!/usr/bin/python3
import itertools, os, subprocess, sys

if __name__ == '__main__':
    VS_API_VERSIONS = ["vapoursynth-api-" + str(v) for v in range(31, 36)]
    VAPOURSYNTH_FUNCTIONS = ["vapoursynth-functions"]
    VSSCRIPT_FUNCTIONS = ["vsscript-functions"]

    if os.environ['TRAVIS_OS_NAME'] == 'osx':
        VSSCRIPT_API_VERSIONS = ["vsscript-api-" + str(v) for v in range(31, 33)]
    else:
        # Trusty VapourSynth is old and doesn't support VSScript API above 3.0.
        VSSCRIPT_API_VERSIONS = []

    features = [VS_API_VERSIONS, VSSCRIPT_API_VERSIONS, VAPOURSYNTH_FUNCTIONS, VSSCRIPT_FUNCTIONS]

    for f in features:
        f += [""]

    someone_failed = False

    for features in itertools.product(*features):
        features_string = str.join(' ', features)
        print("Starting tests with features: " + features_string)
        sys.stdout.flush()

        returncode = subprocess.call(['cargo', 'test', '--quiet', '--features', features_string])
        if returncode != 0:
            someone_failed = True

    if someone_failed:
        sys.exit(1)
