#include "log.hpp"
#include <iostream>
#include <api/api.h>

void log_value(const char* label, int value) {
    std::cout << "[log][" << API_ORIGIN << "] "
              << label << ": " << value << std::endl;
}

