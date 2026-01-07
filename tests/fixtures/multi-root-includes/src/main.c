#include <api/config.h>

#if defined (CONFIG_SOURCE)
int main(void) {
    return CONFIG_SOURCE[0];
}
#else
#error CONFIG_SOURCE not defined
#endif
