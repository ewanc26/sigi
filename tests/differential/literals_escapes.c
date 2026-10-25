#include <stdio.h>

#define EMIT(text) fwrite((text), 1, sizeof(text) - 1, stdout)

int main(void) {
    EMIT("tab\there\n");
    EMIT("quote\" backslash\\ apostrophe\' question\?\n");
    EMIT("octal\101\060\n");
    EMIT("hex\x41\x62\n");
    EMIT("caf\xc3\xa9 é \U0001F600\n");
    EMIT("nul\0byte\n");
    EMIT("bell\a back\b feed\f vertical\v cr\r\n");
    return 0;
}
