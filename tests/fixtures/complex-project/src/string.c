#include "string.h"

int str_length(const char *str) {

#ifdef DEBUG
    #include <stdio.h>
    printf("DEBUG: Computing string lenght for project %s\n", PROJECT_NAME);
#endif
    
    int len = 0;
    while (str[len] != '\0') len++;
    return len;
}
