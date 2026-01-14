#include <iostream>
#include <api/api.h>
#include "shared/shared.h"
#include "log/log.hpp"

int main(void) {
    std::cout << "C++ app\n";
    std::cout << "API origin: " << API_ORIGIN << std::endl;

    int value = shared_compute(5);
    log_value("shared_compute", value);

    return 0;
}
