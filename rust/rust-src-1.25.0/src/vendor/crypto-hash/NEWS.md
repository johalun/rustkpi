# Changes by Version

## Unreleased

## [0.3.0] - 2017-06-18

### Changed

* Upgrade to `commoncrypto` 0.2.x
* Function signatures for `digest` and `hex_digest` changed to use `&[u8]`, per Clippy

## [0.2.1] - 2016-12-12

### Changed

* Move CommonCrypto implementation to its own crate

## [0.2.0] - 2016-11-06

### Added

* SHA-1 algorithm

### Changed

* Upgrade rust-openssl to 0.9

## [0.1.0] - 2016-06-26

This release signifies the minimum amount of algorithms and implementations necessary for
[HTTP digest authentication](https://tools.ietf.org/html/rfc7616).

### Added

Algorithms:

* MD5
* SHA256
* SHA512

Implementations:

* CommonCrypto (OS X)
* CryptoAPI (Windows)
* OpenSSL (Linux/BSD/etc.)

[0.2.1]: https://github.com/malept/crypto-hash/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/malept/crypto-hash/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/malept/crypto-hash/releases/tag/v0.1.0
