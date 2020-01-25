# rweb

[![Build Status](https://travis-ci.com/kdy1/rweb.svg?branch=master)](https://travis-ci.com/kdy1/rweb)

Yet another web server framework for rust.

Installation (without automatic openapi generation):

```toml
[dependencies]
rweb = "0.3.0-alpha.1"
```

# Features

- Safe & Correct

Since `rweb` is based on [warp][], which features safety and correctness, `rweb` has same property.

- Automatic openapi spec generation

# Comparison

| Name    | rweb                                                           | actix-web                                                           | gotham                                                           | iron                                                           | nickel                                                           | rocket                                                           | rouille                                                           | Thruster                                                           | Tide                                                           | tower-web                                                           | warp                                                           |
| ------- | -------------------------------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------- |
| License | ![license](https://img.shields.io/crates/l/rweb.svg?label=%20) | ![license](https://img.shields.io/crates/l/actix-web.svg?label=%20) | ![license](https://img.shields.io/crates/l/gotham.svg?label=%20) | ![license](https://img.shields.io/crates/l/iron.svg?label=%20) | ![license](https://img.shields.io/crates/l/nickel.svg?label=%20) | ![license](https://img.shields.io/crates/l/rocket.svg?label=%20) | ![license](https://img.shields.io/crates/l/rouille.svg?label=%20) | ![license](https://img.shields.io/crates/l/Thruster.svg?label=%20) | ![license](https://img.shields.io/crates/l/tide.svg?label=%20) | ![license](https://img.shields.io/crates/l/tower-web.svg?label=%20) | ![license](https://img.shields.io/crates/l/warp.svg?label=%20) |
| Version | ![version](https://img.shields.io/crates/v/rweb.svg?label=%20) | ![version](https://img.shields.io/crates/v/actix-web.svg?label=%20) | ![version](https://img.shields.io/crates/v/gotham.svg?label=%20) | ![version](https://img.shields.io/crates/v/iron.svg?label=%20) | ![version](https://img.shields.io/crates/v/nickel.svg?label=%20) | ![version](https://img.shields.io/crates/v/rocket.svg?label=%20) | ![version](https://img.shields.io/crates/v/rouille.svg?label=%20) | ![version](https://img.shields.io/crates/v/Thruster.svg?label=%20) | ![version](https://img.shields.io/crates/v/tide.svg?label=%20) | ![version](https://img.shields.io/crates/v/tower-web.svg?label=%20) | ![version](https://img.shields.io/crates/v/warp.svg?label=%20) |

| Github stars | ![github stars](https://img.shields.io/github/stars/kdy1/rweb.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/actix/actix-web.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/gotham-rs/gotham.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/iron/iron.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/nickel-org/nickel.rs.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/SergioBenitez/Rocket.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/tomaka/rouille.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/trezm/Thruster.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/http-rs/tide.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/carllerche/tower-web.svg?label=%20) | ![github stars](https://img.shields.io/github/stars/seanmonstar/warp.svg?label=%20) |

| Contributors | ![contributors](https://img.shields.io/github/contributors/kdy1/rweb.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/actix/actix-web.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/gotham-rs/gotham.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/iron/iron.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/nickel-org/nickel.rs.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/SergioBenitez/Rocket.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/tomaka/rouille.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/trezm/Thruster.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/http-rs/tide.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/carllerche/tower-web.svg?label=%20) | ![contributors](https://img.shields.io/github/contributors/seanmonstar/warp.svg?label=%20) |

| Activity | ![activity](https://img.shields.io/github/commit-activity/m/kdy1/rweb.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/actix/actix-web.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/gotham-rs/gotham.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/iron/iron.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/nickel-org/nickel.rs.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/SergioBenitez/Rocket.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/tomaka/rouille.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/trezm/Thruster.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/http-rs/tide.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/carllerche/tower-web.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/seanmonstar/warp.svg?label=%20) |

| Base framework | hyper / warp | tokio | hyper | hyper | hyper | hyper | tiny-http | tokio | hyper | hyper | hyper |
| https | Y | tokio | hyper | hyper | hyper | hyper | tiny-http | tokio | hyper | hyper | hyper |
| http2 | hyper / warp | tokio | hyper | hyper | hyper | hyper | tiny-http | tokio | hyper | hyper | hyper |

| async | Y | Y | ? | ? | ? | ? | ? | ? | ? | ? | Y (via different method) |
| stable rust | Y | Y | ? | ? | ? | ? | ? | ? | ? | ? | Y |
| openapi support | Y | | | | | | | | | | |

[warp]: https://github.com/seanmonstar/warp
