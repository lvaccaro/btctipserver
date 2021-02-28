BTCTipServer
===
Another Bitcoin payment service, based on [bdk](https://github.com/bitcoindevkit/bdk) and [electrum](https://github.com/bitcoindevkit/rust-electrum-client).

[![build](https://github.com/lvaccaro/btctipserver/workflows/ci/badge.svg)](https://github.com/lvaccaro/btctipserver/actions)
[![MIT license](https://img.shields.io/github/license/lvaccaro/btctipserver)](https://github.com/lvaccaro/btctipserver/blob/master/LICENSE)

### Get it start
Build and run service (default port is 8080):
```
cargo run server
```

Open browser
[localhost:8080](http://localhost:8080) .

### Setup
Configure your wallet parameters in `config.ini` file.

### Deploy on heroku
Fork this project, update `config.ini` file and press the button.

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)
