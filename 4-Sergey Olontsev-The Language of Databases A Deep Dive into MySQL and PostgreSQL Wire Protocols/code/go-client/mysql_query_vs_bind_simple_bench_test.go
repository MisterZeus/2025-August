package main

import (
	"database/sql"
	"log"
	"testing"

	_ "github.com/go-sql-driver/mysql"
)

const mySqlConnectionString = "root:let-me-in@tcp(localhost:3306)/protocols"

func BenchmarkMySqlQuery(b *testing.B) {
	db, err := sql.Open("mysql", mySqlConnectionString)
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
		//if result != 123 {
		//	b.Fatalf("Expected 123, got %d", result)
		//}
	}
}

func BenchmarkMySqlBind(b *testing.B) {
	db, err := sql.Open("mysql", mySqlConnectionString)
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	stmt, err := db.Prepare("select 123 as id")
	if err != nil {
		log.Fatalf("failed to prepare the statement: %v", err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var result int
		err := stmt.QueryRow().Scan(&result)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
		//if result != 123 {
		//	b.Fatalf("Expected 123, got %d", result)
		//}
	}
}
