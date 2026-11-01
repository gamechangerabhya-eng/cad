#!/usr/bin/env python3
"""Verify an Android Rust shared library is a 64-bit ELF for the requested ABI."""
from pathlib import Path
import struct
import sys

EXPECTED = {"arm64-v8a": 183, "x86_64": 62}
NAMES = {183: "AArch64", 62: "x86_64"}


def verify(path: Path, abi: str) -> None:
    if abi not in EXPECTED:
        raise ValueError(f"Unsupported Android ABI: {abi}")
    data = path.read_bytes()
    if len(data) < 64 or data[:4] != b"\x7fELF":
        raise ValueError(f"Not a complete ELF shared library: {path}")
    if data[4] != 2:
        raise ValueError(f"Expected ELF64 library for {abi}: {path}")
    if data[5] != 1:
        raise ValueError(f"Expected little-endian Android ELF for {abi}: {path}")
    elf_type, machine = struct.unpack_from("<HH", data, 16)
    if elf_type != 3:
        raise ValueError(f"Expected ET_DYN shared library, got ELF type {elf_type}: {path}")
    if machine != EXPECTED[abi]:
        raise ValueError(
            f"Wrong ELF machine for {abi}: got {NAMES.get(machine, machine)}, "
            f"expected {NAMES[EXPECTED[abi]]}: {path}"
        )


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: verify-android-elf.py LIBRARY ABI")
    try:
        verify(Path(sys.argv[1]), sys.argv[2])
    except (OSError, ValueError) as exc:
        raise SystemExit(str(exc))
    print(f"Android ELF verified: {sys.argv[2]} ({sys.argv[1]})")
