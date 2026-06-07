// Guards against: Data Class FP.
// A config struct with many fields is the standard Go pattern for application
// configuration — structs are Go's way of grouping related data.

package negative

import "time"

// ServerConfig holds all configuration for an HTTP server.
//
// This is intentionally a plain struct with no methods — Go uses structs
// for configuration, and methods belong on the consumer, not the config.
type ServerConfig struct {
	Host           string
	Port           int
	ReadTimeout    time.Duration
	WriteTimeout   time.Duration
	IdleTimeout    time.Duration
	MaxHeaderBytes int
	TLSCertFile    string
	TLSKeyFile     string
	EnableCORS     bool
	AllowedOrigins []string
	LogFormat      string
	LogLevel       string
}

// DatabaseConfig holds connection settings for a database.
type DatabaseConfig struct {
	Driver          string
	DSN             string
	MaxOpenConns    int
	MaxIdleConns    int
	ConnMaxLifetime time.Duration
	ConnMaxIdleTime time.Duration
}

// RedisConfig holds connection settings for Redis.
type RedisConfig struct {
	Addr     string
	Password string
	DB       int
	PoolSize int
}

// AppConfig aggregates all sub-configurations.
type AppConfig struct {
	Server   ServerConfig
	Database DatabaseConfig
	Redis    RedisConfig
}

// DefaultConfig returns a Config with sensible defaults.
func DefaultConfig() AppConfig {
	return AppConfig{
		Server: ServerConfig{
			Host:           "0.0.0.0",
			Port:           8080,
			ReadTimeout:    15 * time.Second,
			WriteTimeout:   15 * time.Second,
			IdleTimeout:    60 * time.Second,
			MaxHeaderBytes: 1 << 20, // 1 MB
			EnableCORS:     true,
			AllowedOrigins: []string{"*"},
			LogFormat:      "json",
			LogLevel:       "info",
		},
		Database: DatabaseConfig{
			Driver:          "postgres",
			MaxOpenConns:    25,
			MaxIdleConns:    5,
			ConnMaxLifetime: 5 * time.Minute,
			ConnMaxIdleTime: 1 * time.Minute,
		},
		Redis: RedisConfig{
			Addr:     "localhost:6379",
			DB:       0,
			PoolSize: 10,
		},
	}
}
