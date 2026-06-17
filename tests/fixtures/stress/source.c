#define WITH_CALLBACK(name) int name(int value, int (*callback)(int))

typedef struct NestedConfig {
    int timeout_ms;
    struct Limits {
        int retries;
        union {
            int code;
            const char *label;
        } last;
    } limits;
    int (*callback)(int value);
} NestedConfig;

typedef enum Mode {
    ModeCold,
    ModeHot,
} Mode;

static int double_value(int value) {
    return value * 2;
}

WITH_CALLBACK(install_callback) {
    return callback(value);
}

int run_pipeline(NestedConfig *config, Mode mode) {
    int total = 0;
    for (int i = 0; i < config->limits.retries; i++) {
        total += config->callback(i);
    }
    return mode == ModeHot ? install_callback(total, double_value) : total;
}
