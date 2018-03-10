#!/usr/bin/python3
import itertools, os, subprocess, sys

if __name__ == '__main__':
    VS_API_VERSIONS = ["vapoursynth-api-" + str(v) for v in range(31, 36)]
    VAPOURSYNTH_FUNCTIONS = ["vapoursynth-functions"]
    VSSCRIPT_FUNCTIONS = ["vsscript-functions"]

    # The environment variable isn't there on AppVeyor.
    if not 'TRAVIS_OS_NAME' in os.environ or os.environ['TRAVIS_OS_NAME'] == 'osx':
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

        returncode = subprocess.call(['cargo', 'test', '--verbose', '--features', features_string])
        if returncode != 0:
            someone_failed = True
            print("TEST FAILURE: " + features_string)

    if someone_failed:
        print("One of the tests failed, exiting with code 1.")
        sys.exit(1)
