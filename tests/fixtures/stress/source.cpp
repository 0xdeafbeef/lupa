#define SCOPE_EXIT auto scope_exit = [&]

#if ENABLE_CPP_STRESS
namespace engine {

template <class T>
class Pipeline {
public:
    struct Stage {
        T value;
    };

    Pipeline() = default;
    int run(Stage stage) const {
        auto fold = [stage](int seed) { return seed + static_cast<int>(stage.value); };
        return fold(1);
    }
    int configure(int timeout);
};

template <class T>
int Pipeline<T>::configure(int timeout) {
    return timeout + 1;
}

Pipeline<int> make_pipeline() {
    Pipeline<int> pipeline;
    SCOPE_EXIT {
        pipeline.run({0});
    };
    return pipeline;
}

}
#endif
