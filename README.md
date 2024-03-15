BTCTipServer
===
BTCTip Server is a self-hosted, open-source bitcoin payment processor. 

- :coin: use [Bitcoin Development Kit](https://github.com/bitcoindevkit/bdk) a modern, lightweight, descriptor-based wallet library
- :zap: lightning support with [lnsocket](https://github.com/jb55/lnsocket), remote commando plugin
- :droplet: liquid and assets support with [Liquid Wallet Kit](https://github.com/blockstream/lwk)
- :gear: usage wallet descriptors to build complex address rules
- :ninja: keep funds safe without privatekey/secret usage
- :watch: pay host to usage, no required instance always online
- :point_up_2: host on heroku with a click, below button
- :crab: written in rust

![title](assets/preview.png)

### Get it start
Build and run service (default port is 8080):
```
cargo install --path .
btctipserver help
```
Open the local web page on your browser using url [localhost:8080/](http://localhost:8080/).

### Bitcoin
For a bitcoin, pass the descriptor and network (bitcoin or testnet)
```
btctipserver bitcoin --network bitcoin --server "ssl://blockstream.info:700" --descriptor "wpkh(tpubEBr4i6yk5nf5DAaJpsi9N2pPYBeJ7fZ5Z9rmN4977iYLCGco1VyjB9tvvuvYtfZzjD5A8igzgw3HeWeeKFmanHYqksqZXYXGsw5zjnj7KM9/*)"
```

### Liquid
For a liquid, pass the CT descriptor with the master blinding key and network (liquid, liquidtestnet or elements)
```
btctipserver liquid --network liquidtestnet --server "blockstream.info:465" --descriptor "ct(slip77(ab5824f4477b4ebb00a132adfd8eb0b7935cf24f6ac151add5d1913db374ce92),elwpkh([759db348/84'/1'/0']tpubDCRMaF33e44pcJj534LXVhFbHibPbJ5vuLhSSPFAw57kYURv4tzXFL6LSnd78bkjqdmE3USedkbpXJUPA1tdzKfuYSL7PianceqAhwL2UkA/<0;1>/*))#cch6wrnp"
```

### Lightning
For lightning, pass the remote nodeid, the host (with port) and the rune string to access.
```
btctipserver --port "8082" clightning --nodeid "0356ecddb14bf4a12bf1b2e91aadd47b72e37aa81053f2dfa9a2bd7ee928904f30" --host "" --rune ""
```
If your lightning core instance is reachable by onion network, insert the onion endpoint in `host` parameters and add tor socks5 as local proxy as `--proxy "127.0.0.1:9050"`.

### Setup

Pass `--host` and `--port` to specified the host and port to run the server.

If you will be configuring your server via command line args or environment variables then no
changes to the project are needed. If you would like to configure your wallet parameters in a
`config.ini` file then see `config_example.ini` as example.
```
btctipserver -c bitcoin/config.ini bitcoin
```

### Deploy on heroku

1. Fork this project
2. Press the below button
3. When prompted enter required configuration values for your node

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)
