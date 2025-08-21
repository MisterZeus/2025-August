package main

import (
	"database/sql"
	"log"
	"testing"

	_ "github.com/lib/pq"
)

func BenchmarkPgServer(b *testing.B) {
	db, err := sql.Open("postgres", "postgres://postgres:let-me-in@localhost:5432/protocols?sslmode=disable")
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result int
		err := db.QueryRow("select 123 as id").Scan(&result)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
		if result != 123 {
			b.Fatalf("Expected 123, got %d", result)
		}
	}
}

func BenchmarkRustPgEmulationServer(b *testing.B) {
	db, err := sql.Open("postgres", "postgres://postgres:let-me-in@localhost:5433/protocols?sslmode=disable")
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result int
		err := db.QueryRow("select 123 as id").Scan(&result)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
		if result != 123 {
			b.Fatalf("Expected 123, got %d", result)
		}
	}
}
