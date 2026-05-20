typedef struct Config {
    int timeout_ms;
    const char *name;
} Config;

enum Mode {
    MODE_A,
    MODE_B,
};

static int helper(int value) {
    return value + 1;
}

int run_loop(Config *config) {
    return helper(config->timeout_ms);
}
