#!/usr/bin/env python
import sys
import re

def intersect(l1, l2):
    l3 = []
    for elem in l1:
        if elem in l2:
            l3.append(elem)
    return l3

def get_extern_crates(fname):
    with open(fname, "r") as f:
        s = f.read()
        return re.findall('extern crate (\w*);$', s, re.MULTILINE)
        

def main():
    target = sys.argv[1];
    rustmod = sys.argv[2];
    objdir = sys.argv[3];
    modules = sys.argv[4:];
    extcrates = intersect(get_extern_crates(rustmod), modules)

    print ('%s:' % target),
    for extcrate in extcrates:
        print('%s/lib%s.rlib' % (objdir, extcrate)),
    print ''



if __name__ == '__main__':
    main()
