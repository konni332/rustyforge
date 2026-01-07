#include "utils/math.h"
#include "utils/string.h"
#include "network/socket.h"
#include <stdio.h>

int main() {
    Socket s;
    init_socket(&s);

    const char* test_str = "Hello RustyForge!";
    int len = str_length(test_str);

    printf("Test string: '%s' (length %d)\n", test_str, len);

#ifdef DEBUG
    printf("DEBUG: Main executed for project %s\n", PROJECT_NAME);
#endif

    return 0;
}
