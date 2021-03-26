BTCTipServer
===
Another Bitcoin payment service, based on [bdk](https://github.com/bitcoindevkit/bdk) and [electrum](https://github.com/bitcoindevkit/rust-electrum-client).

[![build](https://github.com/lvaccaro/btctipserver/workflows/ci/badge.svg)](https://github.com/lvaccaro/btctipserver/actions)
[![MIT license](https://img.shields.io/github/license/lvaccaro/btctipserver)](https://github.com/lvaccaro/btctipserver/blob/master/LICENSE)
[![bdk](https://raw.githubusercontent.com/bitcoindevkit/bitcoindevkit.org/master/static/badge/bitcoin-dev-kit.svg)](https://github.com/bitcoindevkit/bdk)

### Get it start
Build and run service (default port is 8080):
```
RUST_LOG=info cargo run -- --descriptor "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)"
```

Open the local web page on your browser using url [localhost:8080/bitcoin](http://localhost:8080/bitcoin).

### Setup

If you will be configuring your server via command line args or environment variables then no 
changes to the project are needed. If you would like to configure your wallet parameters in a 
`config.ini` file then see `config_example.ini` as example. 

### Deploy on heroku 

#### Configured via environment vars
1. Fork this project 
2. Press the below button
3. When prompted enter required configuration values for your node

OR 

#### Configured via .ini file 
1. Fork the project
2. Copy `config_default.ini` to `config.ini`
3. Copy `app_ini_example.json` to `app.json`
4. Edit `config.ini` and set configuration values for your node
4. Press the below button

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)
