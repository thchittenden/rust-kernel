#include <stddef.h>

void* memset(char* addr, int val, size_t len) {
    for (size_t i = 0; i < len; i++) {
        addr[i] = val;
    }
    return addr;
}

void* memmove(char* dst, char* src, size_t len) {
    if (dst < src) {
        for (size_t i = 0; i < len; i++) {
            dst[i] = src[i];
        }
    } else {
        int i = len;
        while (i != 0) {
            i--;
            dst[i] = src[i];
        }
    }
    return dst;
}

void* memcpy(char* dst, char* src, size_t len) {
    for (size_t i = 0; i < len; i++) {
        dst[i] = src[i];
    }
    return dst;
}

int memcmp(char* p1, char* p2, size_t len) {
    for (size_t i = 0; i < len; i++) {
        char a = p1[i];
        char b = p2[i];
        if (a != b) {
            return a - b;
        }
    }
    return 0;
}

