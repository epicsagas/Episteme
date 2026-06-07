// Guards against: Middle Man FP, Feature Envy FP.
// The io.Reader decorator adds cross-cutting logging around each Read call.
// Delegation to the inner reader is the point of the decorator pattern.

package negative

import (
	"io"
	"log"
	"time"
)

// LoggingReader wraps an io.Reader and logs each Read call's byte count
// and duration. The delegation to inner.Read is intentional — this is
// a decorator, not mindless forwarding.
type LoggingReader struct {
	inner  io.Reader
	logger *log.Logger
}

// NewLoggingReader creates a LoggingReader that logs read activity.
func NewLoggingReader(inner io.Reader, logger *log.Logger) *LoggingReader {
	return &LoggingReader{inner: inner, logger: logger}
}

// Read implements io.Reader, logging the number of bytes read and elapsed time.
func (r *LoggingReader) Read(p []byte) (int, error) {
	start := time.Now()
	n, err := r.inner.Read(p)
	elapsed := time.Since(start)
	r.logger.Printf("read %d bytes in %v (err=%v)", n, elapsed, err)
	return n, err
}

// MeasuringReader wraps an io.Reader and counts total bytes read.
type MeasuringReader struct {
	inner    io.Reader
	BytesRead int64
}

// NewMeasuringReader creates a reader that tracks total bytes read.
func NewMeasuringReader(inner io.Reader) *MeasuringReader {
	return &MeasuringReader{inner: inner}
}

// Read implements io.Reader, accumulating the byte count.
func (r *MeasuringReader) Read(p []byte) (int, error) {
	n, err := r.inner.Read(p)
	r.BytesRead += int64(n)
	return n, err
}
