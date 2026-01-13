#include <assert.h>
#include "shared/shared.h"

int main(void) {
    int v = shared_compute(1);
    assert(v > 0);
    return 0;
}
