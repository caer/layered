FROM fedora:43

# Common development packages.
RUN dnf install -y awk curl wget hostname git gcc g++ cmake

# Common customizations.
COPY .bashrc /root/.bashrc

# Rust toolchain.
RUN dnf install -y rustup
RUN rustup-init -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add wasm32-unknown-unknown

# Rust utilities.
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --no-confirm cargo-audit cargo-server

# Macroquad support packages.
RUN dnf install -y libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel