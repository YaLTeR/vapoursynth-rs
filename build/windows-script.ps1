# Get the arch and dir
param([String]$arch="x86_64")

$NAME = "VapourSynth"
$PY_DIR = "py-dir"
$VS_DIR = "vs-dir"

If ($arch -eq "i686") {
        $SUFFIX = 32
        $PYTHON_PKG = "python-3.9.10-embed-win32.zip"
} Else {
        $SUFFIX = 64
        $PYTHON_PKG = "python-3.9.10-embed-amd64.zip"
}

# Download Python embeddable and VapourSynth portable
$VS_PATH = "https://github.com/vapoursynth/vapoursynth/releases/download/R57"
curl -LO "https://www.python.org/ftp/python/3.9.10/$PYTHON_PKG"
curl -LO "$VS_PATH/VapourSynth$SUFFIX-Portable-R57.7z"

# Unzip Python embeddable and VapourSynth portable
7z x "$PYTHON_PKG" -o"$PY_DIR"
7z x "VapourSynth$SUFFIX-Portable-R57.7z" -o"$VS_DIR"

# Move all VapourSynth files inside the Python ones
Move-Item -Force -Path "$VS_DIR\*" -Destination "$PY_DIR"

# Move the VapourSynth directory into a system directory
Move-Item -Path "$PY_DIR" -Destination "C:\Program Files"
Rename-Item -Path "C:\Program Files\$PY_DIR" -NewName "$NAME"
