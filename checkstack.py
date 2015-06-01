#!/usr/bin/env python
import sys
import re

def main():
    minsize = int(sys.argv[1])
    stdin = sys.stdin.read();
    res = re.findall('^\s*([a-fA-F0-9]+).*?sub\s*\$(0x[a-fA-F0-9]+)\s*,\s*%esp',stdin, re.M)
    res = map(lambda (addr, size): (addr, int(size, 16)), res)
    res = filter(lambda (addr, size): size > minsize, res)
    res.sort(key=lambda (addr, size): size)
    if len(res) == 0:
        print 'PASSED stack check'
    else:
        for (addr, size) in res:
            print 'WARNING: stack frame of size %4d at 0x%s' % (size, addr)
    print ''

if __name__ == '__main__':
    main()
