# <img src="https://storage.googleapis.com/iqlusion-production-web/github/delphi/delphi-logo.svg" width="450px" alt="Sagan">

[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![MSRV][msrv-image]
[![Apache 2.0 Licensed][license-image]][license-link]
[![Gitter Chat][gitter-image]][gitter-link]

## About
Oracle feeder service (presently supporting [Terra]). Currently `delphi` is used in production by iqlusion and has been integrated with [Tendermint KMS' transaction signer][tmkms]. Detailed [architecture available here][tmkms_batch]. 

Terra 🌏🔗:
- [Oracle Feeder docs][terra_oracle]  
- [iqlusion aggregate exchange rate votes][iqlusion_stakeid]


## Sources
Following exchanges are supported: 
- [Alpha Vantage][alphavantage]
- [Binance][binance]
- [Coinone][coinone]
- [Dunamu][dunamu]
- [GDAC][gdac]
- [GOPAX][gopax]
- [IMF SDR][imfsdr]
- [IMF SDR][imfsdr]
- [Currencylayer][currencylayer]

### Alpha Vantage 
This source requires an API key. Request key from [Alpha Vantage][alphavantage_api] then add to following config file. 

### Currencylayer
This source requires an API key. Request key from [Currencylayer][currencylayer_api] then add to following config file. 

### Config
Create config with `touch delphi.toml` then add your relevant network configuration.
```
# Example Delphi configuration file

# Listen address configuration
[listen]
addr = "127.0.0.1"
port = 3822
protocol = "http"

# HTTPS client configuration
# [https]
# proxy = "https://webproxy.example.com:8080" # send outgoing requests through proxy

# Network configuration: blockchains for which oracle service is provided
[network.terra]
chain_id = "columbus-4"
feeder = "terra1..."
validator = "terravaloper1..."
fee = { denom = "Ukrw", amount = "356100", gas = "200000" }

# Source configuration: exchanges where price information is gathered from
[source.alphavantage]
# Get API key here (quick-and-simple form): https://www.alphavantage.co/support/#api-key
apikey = "api key goes here"
```


## Operating Systems
- Linux (recommended)

## Code of Conduct

We abide by the [Contributor Covenant][cc] and ask that you do as well.

For more information, please see [CODE_OF_CONDUCT.md].

## License

Copyright © 2019 iqlusion

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[//]: # (badges)

[build-image]: https://github.com/iqlusioninc/delphi/workflows/Rust/badge.svg?branch=main&event=push
[build-link]: https://github.com/iqlusioninc/delphi/actions
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[msrv-image]: https://img.shields.io/badge/rustc-1.44+-blue.svg
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/iqlusioninc/delphi/blob/master/LICENSE
[gitter-image]: https://badges.gitter.im/badge.svg
[gitter-link]: https://gitter.im/iqlusioninc/community

[//]: # (general links)

[alphavantage]: https://www.alphavantage.co/
[alphavantage_api]: https://www.alphavantage.co/support/#api-key
[currencylayer]: https://www.currencylayer.com
[currencylayer_api]: https://currencylayer.com/product
[binance]: https://binance-docs.github.io/apidocs/spot/en/#change-log
[cc]: https://contributor-covenant.org
[CODE_OF_CONDUCT.md]: https://github.com/iqlusioninc/delphi/blob/main/CODE_OF_CONDUCT.md
[coinone]: https://doc.coinone.co.kr/
[dunamu]: https://www.dunamu.com/
[gdac]: https://docs.gdac.com/#introduction
[gopax]: https://www.gopax.co.id/API/
[imfsdr]: https://www.imf.org/external/index.htm
[iqlusion_stakeid]: https://terra.stake.id/?#/validator/EA2D131F0DE4A91CC7ECA70FAAEB7F088F5DC6C3
[Terra]: https://terra.money/
[terra_oracle]: https://docs.terra.money/validator/oracle.html
[tmkms]: https://github.com/iqlusioninc/tmkms/blob/main/README.txsigner.md
[tmkms_batch]: https://github.com/iqlusioninc/tmkms/blob/main/README.txsigner.md#architecture
