#include <stdio.h>
#include <api/api.h>
#include "api/version.h"
#include "config/build_config.h"
#include "shared/shared.h"

int main(void) {
    int unused_binding;
    printf("C app\n");

    #ifdef RF_PROFILE_RELEASE
        printf("Profile: release\n");
    #endif

    printf("API origin: %s\n", API_ORIGIN);
    printf("API version: %d.%d\n", API_VERSION_MAJOR, API_VERSION_MINOR);
    printf("Build mode: %s\n", BUILD_MODE);
    printf("Result: %d\n", shared_compute(10));
    return 0;
}
