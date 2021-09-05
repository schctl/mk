#if defined(__linux__)
    #include <crypt.h>
#else
    #include <unistd.h>
#endif
