package main

import (
	"database/sql"
	"fmt"
	"testing"

	_ "github.com/go-sql-driver/mysql"
)

const (
	dbUserWithoutCompression = "root:let-me-in@tcp(localhost:3306)/protocols"
	dbUserWithCompression    = "root:let-me-in@tcp(localhost:3306)/protocols?compress=true"
)

var (
	dbWithoutCompression *sql.DB
	dbWithCompression    *sql.DB
)

func setupDatabases() error {
	var err error

	// Connection without compression
	dbWithoutCompression, err = sql.Open("mysql", dbUserWithoutCompression)
	if err != nil {
		return fmt.Errorf("failed to connect without compression: %v", err)
	}

	dbWithoutCompression.SetMaxOpenConns(10)
	dbWithoutCompression.SetMaxIdleConns(5)

	if err = dbWithoutCompression.Ping(); err != nil {
		return fmt.Errorf("failed to ping without compression: %v", err)
	}

	// Connection with compression
	dbWithCompression, err = sql.Open("mysql", dbUserWithCompression)
	if err != nil {
		return fmt.Errorf("failed to connect with compression: %v", err)
	}

	dbWithCompression.SetMaxOpenConns(10)
	dbWithCompression.SetMaxIdleConns(5)

	if err = dbWithCompression.Ping(); err != nil {
		return fmt.Errorf("failed to ping with compression: %v", err)
	}

	return nil
}

func closeDatabases() {
	if dbWithoutCompression != nil {
		dbWithoutCompression.Close()
	}
	if dbWithCompression != nil {
		dbWithCompression.Close()
	}
}

func BenchmarkWithCompression(b *testing.B) {
	if dbWithCompression == nil {
		b.Skip("Database not initialized")
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result string
		err := dbWithCompression.QueryRow("select repeat('a', 1000)").Scan(&result)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
	}
}

func BenchmarkWithoutCompression(b *testing.B) {
	if dbWithoutCompression == nil {
		b.Skip("Database not initialized")
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result string
		err := dbWithoutCompression.QueryRow("select repeat('a', 1000)").Scan(&result)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
	}
}

func BenchmarkWithoutCompressionParallel(b *testing.B) {
	if dbWithoutCompression == nil {
		b.Skip("Database not initialized")
	}

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			var result string
			err := dbWithoutCompression.QueryRow("select repeat('a', 1000)").Scan(&result)
			if err != nil {
				b.Errorf("Query failed: %v", err)
				return
			}
		}
	})
}

func BenchmarkWithCompressionParallel(b *testing.B) {
	if dbWithCompression == nil {
		b.Skip("Database not initialized")
	}

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			var result string
			err := dbWithCompression.QueryRow("select repeat('a', 1000)").Scan(&result)
			if err != nil {
				b.Errorf("Query failed: %v", err)
				return
			}
		}
	})
}

func TestMain(m *testing.M) {
	if err := setupDatabases(); err != nil {
		fmt.Printf("Failed to setup databases: %v\n", err)
		return
	}

	code := m.Run()

	closeDatabases()

	if code != 0 {
		panic("Tests failed")
	}
}
