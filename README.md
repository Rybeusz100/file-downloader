# Download files directly onto your server

## General info
This project provides functionality to download files directly onto the machine it's running on, for example your home server. Supports multiple users.

### Supported URL types
- WeTransfer (both normal and from emails)
- YouTube (uses rustube crate and is very slow)

## Setup
### Requirements
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org)
- [Docker](https://www.docker.com) (optional)

## Docker setup
```
docker build -t file-downloader .
```
Create a container mapping the following values:
- TCP port 8055
- /config
- /downloads

To create a new user make a POST request to `SERVER_IP:8055/create_user` with payload:
```json
{
    "name": "your-username",
    "password": "your-password"
}
```
