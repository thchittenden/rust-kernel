#!/usr/bin/env python
import sys
import re

def main():
    stdin = sys.stdin.read();
    sizes = re.findall('^\s*\d+:\s*([A-Fa-f0-9]+)\s*\d\s*NOTYPE\s*GLOBAL\s*DEFAULT\s*ABS\s*_binary\w*_(\w*)_size', stdin, re.M)
    sizes.sort(key=lambda (size, name): name)
    addrs = re.findall('^\s*\d+:\s*([A-Fa-f0-9]+)\s*\d\s*NOTYPE\s*GLOBAL\s*DEFAULT\s*1\s*_binary\w*_(\w*)_start', stdin, re.M)
    addrs.sort(key=lambda (addr, name): name)

    for ((addr, name1), (size, name2)) in zip(addrs, sizes):
        assert(name1 == name2)
        sys.stdout.write('{}\0{}\0{}\0'.format(name1, addr, size))
    sys.stdout.write('\0')

if __name__ == '__main__':
    main()
