# UltiVM
A little CollabVM clone written in Rust. (buggy tho)
## Building
Before building, make sure you have Rust with Cargo installed. Also, Git is required to clone the repo. Many commands use Unix syntax because I use Linux.
### Steps
1. Clone the repo and go to it's directory
```bash
git clone https://github.com/imguest24897-alt/UltiVM.git ; cd UltiVM
```
2. Get some of the submodules
```bash
git submodule update --init --recursive
```
3. Installing dependencies (if your cargo does not want to do that)
```bash
cargo install actix-web
cargo install config
cargo install serde
```
4. Build it
```bash
cargo build --path .
```
If you want a release build, you can use this command:
```bash
cargo build --release
```
## Launching after a build
If you are confused where it is, it's at `target/debug`. If you did a release build, it would be `target/release`.