// Guards against: Long Parameter List FP.
// Go's functional options pattern replaces long parameter lists with
// variadic Option functions — this is the idiomatic solution to the problem.

package negative

import "time"

// Server is configured via functional options.
type Server struct {
	host         string
	port         int
	timeout      time.Duration
	maxConns     int
	readTimeout  time.Duration
	writeTimeout time.Duration
	shutdownWait time.Duration
}

// Option is a function that configures a Server.
type Option func(*Server)

// WithPort sets the server port.
func WithPort(port int) Option {
	return func(s *Server) { s.port = port }
}

// WithTimeout sets the overall request timeout.
func WithTimeout(d time.Duration) Option {
	return func(s *Server) { s.timeout = d }
}

// WithMaxConnections sets the maximum number of concurrent connections.
func WithMaxConnections(n int) Option {
	return func(s *Server) { s.maxConns = n }
}

// WithReadTimeout sets the read timeout.
func WithReadTimeout(d time.Duration) Option {
	return func(s *Server) { s.readTimeout = d }
}

// WithWriteTimeout sets the write timeout.
func WithWriteTimeout(d time.Duration) Option {
	return func(s *Server) { s.writeTimeout = d }
}

// WithShutdownGrace sets the graceful shutdown wait period.
func WithShutdownGrace(d time.Duration) Option {
	return func(s *Server) { s.shutdownWait = d }
}

// NewServer creates a Server with the given options applied.
// Uses defaults for any option not specified.
func NewServer(opts ...Option) *Server {
	s := &Server{
		host:         "0.0.0.0",
		port:         8080,
		timeout:      30 * time.Second,
		maxConns:     1000,
		readTimeout:  15 * time.Second,
		writeTimeout: 15 * time.Second,
		shutdownWait: 10 * time.Second,
	}
	for _, opt := range opts {
		opt(s)
	}
	return s
}

// Addr returns the server listen address.
func (s *Server) Addr() string {
	return s.host
}

// Port returns the server port.
func (s *Server) Port() int {
	return s.port
}
