# sentry-breakdown
Render Sentry project breakdown of usage report (WIP)

## Prerequisites
Install [wasm-pack](https://github.com/rustwasm/wasm-pack)

## How to build and run
```
% wasm-pack build --target web --release
[INFO]: Checking for the Wasm target...
[INFO]: Compiling to Wasm...
    Finished release [optimized] target(s) in 0.02s
[INFO]: Installing wasm-bindgen...
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
[INFO]: :-) Done in 6.07s
[INFO]: :-) Your wasm pkg is ready to publish at /home/eagletmt/.clg/github.com/eagletmt/sentry-breakdown/pkg.
% docker run --publish 80:80 -v "$PWD:/usr/share/nginx/html:ro" public.ecr.aws/nginx/nginx:mainline
```
