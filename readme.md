# recore

A simple RISC-V kernel, which could be seen as a incomplete reimplementation of [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3).

## Features

- concurrency of multiple processes, each of which contains mutiple native threads
- dynamic memory management in kernel supported by SLUB
- virtual memory with SV39
- a simple file system with a block cache
- an interactive shell in the userspace that supports some basic commands

## Run

To run this kernel, you should:

``` bash
$ git clone git@github.com:Celve/recore.git
$ cd recore/kernel
$ make fsimg
$ make run
```

There should be an interactive shell that provide similar interface like the normal shell on your operating system. Use it to explore the functionality of this simple OS.

Supported shell commands are listed in `user/src/bin`, or use `ls` in shell to list.

## Wiki

For more details, please check the [wiki](https://github.com/Celve/recore/wiki).