package main

import (
	"database/sql"
	"log"
	"testing"

	_ "github.com/go-sql-driver/mysql"
)

func BenchmarkMySqlServer(b *testing.B) {
	db, err := sql.Open("mysql", "root:let-me-in@tcp(localhost:3306)/protocols")
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

func BenchmarkRustMySqlEmulationServer(b *testing.B) {
	db, err := sql.Open("mysql", "root:let-me-in@tcp(localhost:3307)/protocols")
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

//func BenchmarkMySqlServerParallel(b *testing.B) {
//	db, err := sql.Open("mysql", "root:let-me-in@tcp(localhost:3306)/protocols")
//	if err != nil {
//		log.Fatalf("failed to connect without compression: %v", err)
//	}
//	defer db.Close()
//
//	db.SetMaxOpenConns(10)
//	db.SetMaxIdleConns(5)
//
//	b.RunParallel(func(pb *testing.PB) {
//		for pb.Next() {
//			var result int
//			err := db.QueryRow("select 123 as id").Scan(&result)
//			if err != nil {
//				b.Fatalf("Query failed: %v", err)
//			}
//			if result != 123 {
//				b.Fatalf("Expected 123, got %d", result)
//			}
//		}
//	})
//}
//
//func BenchmarkRustMySqlEmulationServerParallel(b *testing.B) {
//	db, err := sql.Open("mysql", "root:let-me-in@tcp(localhost:3307)/protocols")
//	if err != nil {
//		log.Fatalf("failed to connect without compression: %v", err)
//	}
//	defer db.Close()
//
//	db.SetMaxOpenConns(10)
//	db.SetMaxIdleConns(5)
//
//	b.RunParallel(func(pb *testing.PB) {
//		for pb.Next() {
//			var result int
//			err := db.QueryRow("select 123 as id").Scan(&result)
//			if err != nil {
//				b.Fatalf("Query failed: %v", err)
//			}
//			if result != 123 {
//				b.Fatalf("Expected 123, got %d", result)
//			}
//		}
//	})
//}
