# DoNV
Do (ดู) Nvidia gpu

## Requirements

- Rust (latest stable recommended)
- NVIDIA drivers installed
- NVML available on your system (usually comes with NVIDIA drivers)
- Compatible with Linux systems using NVIDIA GPUs

### Usage

```bash
cargo run 

# or
cargo build --release

# To run the binary, current directory of binary should have the static/ and its content. 
# You can also move the binary to the root of the project.

mv target/release/donv .
./donv  

```

The website can be access from `http://localhost:26501`.

![](https://media.discordapp.net/attachments/1056861101758885920/1361035543739240659/image.png?ex=67fd4aa0&is=67fbf920&hm=af08ec505319f3ca20666ea51b4939f688e6d9a5c285512731540a750c05113e&=&format=webp&quality=lossless&width=1024&height=635)