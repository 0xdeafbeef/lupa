#if ENABLE_C_STRESS
#define WITH_CALLBACK(name) int name(int value, int (*callback)(int))
#define ASSERT_VALUE(expected, op, actual) ((expected)op(actual))

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
    ASSERT_VALUE(0, <=, total);
    ASSERT_VALUE(total,
                 >, -1);
    return mode == ModeHot ? install_callback(total, double_value) : total;
}
#endif
