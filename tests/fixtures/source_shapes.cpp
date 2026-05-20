namespace net {
class Client {
public:
    Client();
    ~Client();
    int connect(int timeout);
    bool operator==(const Client &other) const;

private:
    int fd_;
};

Client::Client() = default;

Client::~Client() = default;

int Client::connect(int timeout) {
    return timeout + fd_;
}

bool Client::operator==(const Client &other) const {
    return fd_ == other.fd_;
}

template <class T>
T identity(T value) {
    return value;
}
}
