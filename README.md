[![kbw](https://snapcraft.io//kbw/badge.svg)](https://snapcraft.io/kbw)

# Ket Bitwise Simulator

Ket Bitwise Simulator (KBW) server is the quantum computer simulator of the Ket
Quantum Programming. The simulator executes Ket Quantum Assembly (.kqasm)
generated by the libket (and the Ket quantum programming language), using the
bitwise representation [[arxiv:2004.03560](https://arxiv.org/abs/2004.03560)]. 

## Usage

```shell
$ kbw -h
Ket Biswise Simulator server
============================

usage: kbw [-h] [-b 127.0.1.1] [-p 4242] [-s random] [-l]

Ket Biswise Simulator server

optional arguments:
  -h, --help     show this help message and exit
  -b 127.0.1.1   Server bind
  -p 4242        Server port
  -s random      Seed for the PRNG
  -l             Extra plugin path
```

## Installation

The kbw is available in most Linux distribution through the Snap Store.

> Information on how to enable Snap on your Linux distribution is available on
> https://snapcraft.io/kbw.

```shell
sudo snap install kbw --edge
```

### Install from source 

To install from source, follow the commands:

```shell
git clone https://gitlab.com/quantum-ket/kbw.git
cd kbw
python setup.py install --user
```

-----------

This project is part of the Ket Quantum Programming, see the documentation for
more information https://quantum-ket.gitlab.io.