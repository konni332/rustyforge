#include "core.h"
#include "internal/core_internal.h"
#include "util/util.h"

int core_compute(int x) {
    return util_scale(x) + CORE_INTERNAL_MAGIC;
}
