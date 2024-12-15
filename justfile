default:
  just -l

new-window NAME CMD:
  tmux new-w -t midi-stepper -n "{{NAME}}"
  tmux send-keys -t midi-stepper:"{{NAME}}" ". ./.venv/bin/activate" ENTER
  tmux send-keys -t midi-stepper:"{{NAME}}" "{{CMD}}" ENTER

tmux:
  tmux new -ds midi-stepper -n "README"
  tmux send-keys -t midi-stepper:README 'nv ./README.md "+set wrap"' ENTER
  @just new-window "GUI" "nv ./gui/Stepper-Synth.pygame +'setfiletype python'"
  @just new-window "Edit" ""
  @just new-window "Run" ""
  @just new-window "Git" "git status"
  @just new-window "Misc" ""
  tmux a -t midi-stepper

install-lib:
  pip uninstall -y stepper-synth-backend && maturin develop

only-run:
  python ./gui/Stepper-Synth.pygame

run-new: install-lib only-run

hardware-test: build-debug flash-adb

flash-hardware: build-release flash-adb

hardware-errors:
  adb shell cat /userdata/roms/ports/stepper-synth/out.txt

hardware-reboot:
  adb reboot

flash-adb:
  adb shell "mkdir /userdata/roms/ports/stepper-synth/"
  adb push ./{gui/{Stepper-Synth.pygame,stepper_synth,Anonymous-Pro.ttf},dist/stepper_synth_backend-0.1.0-cp311-cp311-manylinux_2_17_aarch64.manylinux2014_aarch64.whl} /userdata/roms/ports/stepper-synth/
  adb shell "cd /userdata/roms/ports/stepper-synth/; .venv/bin/python -m pip install --force-reinstall --no-index ./stepper_synth_backend-*aarch64.whl"

build-debug:
  PKG_CONFIG_SYSROOT_DIR=./cross-build-deps/aarch64 maturin build --out dist --find-interpreter --target aarch64-unknown-linux-gnu --zig

build-release:
  PKG_CONFIG_SYSROOT_DIR=./cross-build-deps/aarch64 maturin build --out dist --find-interpreter --target aarch64-unknown-linux-gnu --zig --release

force-kill-synth:
  adb shell "killall python"

run-on-device:
  adb shell "/userdata/roms/ports/stepper-synth/stepper-synth.sh"

set-hardware-time:
  adb shell "date $(date +%m%d%H%M%Y.%S)"

setup-aarch64:
  mkdir -p ./cross-build-deps/aarch64/
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/systemd-libs-256.7-1-aarch64.pkg.tar.xz 
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/gcc-libs-14.1.1+r1+g43b730b9134-1-aarch64.pkg.tar.xz
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/glibc-2.39+r52+gf8e4623421-1-aarch64.pkg.tar.xz
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/linux-api-headers-6.10-1-aarch64.pkg.tar.xz
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/python-3.12.7-1-aarch64.pkg.tar.xz
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/core/libcap-2.71-1-aarch64.pkg.tar.xz
  # wget -nv -P ./cross-build-deps/aarch64/ http://mirror.archlinuxarm.org/aarch64/extra/alsa-lib-1.2.13-1-aarch64.pkg.tar.xz
  cd ./cross-build-deps/aarch64; for f in $(ls *.pkg.tar.xz); do echo "extracting archiver: $f"; tar xf $f && rm $f; done
