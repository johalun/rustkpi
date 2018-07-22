bufstream
=========

Buffered I/O streams for reading/writing

[![Build Status](https://travis-ci.org/alexcrichton/bufstream.svg?branch=master)](https://travis-ci.org/alexcrichton/bufstream)

[Documentation](http://alexcrichton.com/bufstream)

## Usage

```toml
[dependencies]
bufstream = "0.1"
```

## Tokio

There is support for tokio's `AsyncRead` + `AsyncWrite` traits through the `tokio`
feature. When using this crate with asynchronous IO, make sure to properly flush
the stream before dropping it since IO during drop may cause panics. For the same
reason you should stay away from `BufStream::into_inner`.
