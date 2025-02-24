# File share
Cross Platform GUI software to share files in local network or over the internet. 
The Programm acts as an http server. Clients can download the selected files over a web browser.

Written in Rust using [Iced](https://github.com/iced-rs/iced). 
Upload | Downlaod
-----------------|--------------------------
![image](https://github.com/user-attachments/assets/5de6d0a3-a54a-46b0-9a31-e9095109c7bf) | ![image](https://github.com/user-attachments/assets/42817b68-60be-458d-8691-0338bbacb6e6)
![image](https://github.com/user-attachments/assets/3267d1a5-23c6-43da-bcab-7b52dbfa0e7a) | ![image](https://github.com/user-attachments/assets/82782f36-c161-46a8-bc0b-0b7466f4af29)

## Features
* Share files over the local network or over the internet 
* Zip folders before sharing (optional)
* Support for multiple themes and languages

### Run with
```
cargo run --release
```

### Optimized Build
```
cargo run --profile optimized --features appdata
```
The feature *appdata* configures the programm to write config files in the appdata folder. 
