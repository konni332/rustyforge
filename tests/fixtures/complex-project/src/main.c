#include <stdio.h>
#include <api/api.h>
#include "api/version.h"
#include "config/build_config.h"
#include "shared/shared.h"

int main(void) {
    printf("C app\n");
    printf("API origin: %s\n", API_ORIGIN);
    printf("API version: %d.%d\n", API_VERSION_MAJOR, API_VERSION_MINOR);
    printf("Build mode: %s\n", BUILD_MODE);
    printf("Result: %d\n", shared_compute(10));
    return 0;
}
