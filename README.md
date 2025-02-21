# File share
Cross Platform GUI software to share files in local network or over the internet. 
The Programm acts as an http server. Clients can download the selected files over a web browser.

Written in Rust using [Iced](https://github.com/iced-rs/iced). 

### Run with
```
cargo run --release
```

### Optimized Build
```
cargo run --profile optimized --features appdata
```
The feature *appdata* configures the programm to write config files in the appdata folder. 
