"""Write a 1024x1024 solid-color RGBA PNG (no third-party modules)."""
import struct
import sys
import zlib

path = sys.argv[1]
w = h = 1024
r, g, b, a = 0x2E, 0x34, 0x40, 0xFF
row = b"\x00" + bytes([r, g, b, a]) * w
raw = row * h

def chunk(tag, data):
    c = struct.pack(">I", len(data)) + tag + data
    return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

png = b"\x89PNG\r\n\x1a\n"
png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(raw, 9))
png += chunk(b"IEND", b"")
with open(path, "wb") as f:
    f.write(png)
