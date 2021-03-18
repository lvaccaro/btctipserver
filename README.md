BTCTipServer
===
Another Bitcoin payment service, based on [bdk](https://github.com/bitcoindevkit/bdk) and [electrum](https://github.com/bitcoindevkit/rust-electrum-client).

[![build](https://github.com/lvaccaro/btctipserver/workflows/ci/badge.svg)](https://github.com/lvaccaro/btctipserver/actions)
[![MIT license](https://img.shields.io/github/license/lvaccaro/btctipserver)](https://github.com/lvaccaro/btctipserver/blob/master/LICENSE)

### Get it start
Build and run service (default port is 8080):
```
RUST_LOG=info cargo run
```

Open the local web page on your browser using url [localhost:8080/bitcoin](http://localhost:8080/bitcoin).

### Setup
Configure your wallet parameters in `config.ini` file. 
See `config_example.ini` as example.

### Deploy on heroku
1. Fork this project 
2. Copy `config_default.ini` to `config.ini`
3. Setup your node with `config.ini` file
4. Press the following button

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)
