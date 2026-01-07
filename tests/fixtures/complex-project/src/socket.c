#include "socket.h"
#include <stdio.h>

void init_socket(Socket *s) {
    // Trigger warning:
    int unused_variable;

    s->fd = 0;
    s->name = PROJECT_NAME;

    printf("Socket initialized: %s\n", s->name);
}
