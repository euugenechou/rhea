# rhea

A QEMU-based virtual machine manager.
Written mostly because remembering/typing QEMU commands is hard.

## Installation

Requires [Rust/Cargo](https://www.rust-lang.org/tools/install).

```bash
git clone https://github.com/euugenechou/rhea.git
cd rhea
cargo install --path .
```

## Settings

`rhea` requires the environment variable `RHEA_UEFI_PATH` to be set as the path
to the UEFI blob to use. On an M1 Mac, assuming QEMU is installed using
[`brew`](https://brew.sh), this should be set as:

```bash
export RHEA_UEFI_PATH="$(brew --prefix qemu)/share/qemu/edk2-aarch64-code.fd"
```

If you're on an Intel-based Mac, simply use UEFI blob designated for x86-64. If
not on a Mac, the blobs will (probably) require some digging to locate.

## Usage

See program help for usage.

```bash
rhea help
```

## Notes

This was designed by me for use by me, so no guarantees that nothing will break.
