#include <foo/foo.h>

int foo(void) {
    return COMMON_VALUE + BAR_VALUE;
}

int main(void) {
    return foo();
}
