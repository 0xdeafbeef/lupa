package fixtures

import "context"

type Server struct {
	name string
	Handler
	clock Clock
}

type Handler interface {
	Handle(ctx context.Context) error
	Close() error
}

type Clock interface {
	Now() int64
}

type Alias = Handler

func NewServer(name string, handler Handler) *Server {
	return &Server{name: name, Handler: handler}
}

func (s *Server) Start(ctx context.Context) error {
	return s.Handler.Handle(ctx)
}

func (s Server) stop() {
}

func helper() {}
