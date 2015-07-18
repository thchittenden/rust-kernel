#include "syscalls.h"

extern int main();

void _start() {
    vanish(main());
}
