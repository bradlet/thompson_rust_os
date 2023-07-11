# thompson_rust_os
Working through https://os.phil-opp.com/ for CS 506 at Portland State University - Summer 2023.

## Build
This project needs to be built for a bare-metal target environment.

> Note: Following target env is temporary until we setup a custom target.

First, add the compile target via rustup:
> rustup target add thumbv7em-none-eabihf

Now you can build for that env:
> cargo build --target thumbv7em-none-eabihf