#!/usr/bin/env python
import sys
import re

def main():
    stdin = sys.stdin.read();
    res = re.findall('^\s*\d+:\s*([A-Fa-f0-9]+)\s*\d\s*NOTYPE\s*GLOBAL\s*DEFAULT\s*1\s*_binary_obj_progs_(\w*)_start', stdin, re.M)
    for (addr, name) in res:
        sys.stdout.write('{}\0{}\0'.format(name, addr))
    sys.stdout.write('\0')

if __name__ == '__main__':
    main()
