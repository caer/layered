#!/bin/bash

# mkdir -p target/tmp
# wget https://raw.githubusercontent.com/not-fl3/miniquad/refs/heads/master/js/gl.js -O target/tmp/gl.js
# wget https://raw.githubusercontent.com/not-fl3/quad-snd/refs/heads/master/js/audio.js -O target/tmp/audio.js
# wget https://raw.githubusercontent.com/not-fl3/quad-net/refs/heads/master/js/quad-net.js -O target/tmp/quad-net.js
# wget https://raw.githubusercontent.com/not-fl3/sapp-jsutils/refs/heads/master/js/sapp_jsutils.js -O target/tmp/sapp_jsutils.js

# function wrap_js {
#     echo "(function () {" >> web/macroquad.js
#     cat $1 >> web/macroquad.js
#     echo "}());" >> web/macroquad.js
# }

# echo -n "" > web/macroquad.js
# cat target/tmp/gl.js > web/macroquad.js
# wrap_js target/tmp/sapp_jsutils.js
# wrap_js target/tmp/audio.js
# wrap_js target/tmp/quad-net.js

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/layered.wasm web/layered.wasm
cargo server --path web/