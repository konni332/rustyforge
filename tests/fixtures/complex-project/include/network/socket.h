#pragma once

#include "../common.h"

typedef struct {
    int fd;
    const char* name;
} Socket;

void init_socket(Socket* s);
