package main

import (
	"database/sql"
	"log"
	"testing"

	_ "github.com/go-sql-driver/mysql"
)

type product struct {
	title       string
	description sql.NullString
	categoryId  sql.NullInt16
}

const mySqlConnectionString = "root:let-me-in@tcp(localhost:3306)/protocols"

func BenchmarkMySqlComplexQuery(b *testing.B) {
	db, err := sql.Open("mysql", mySqlConnectionString)
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var p product
		err := db.QueryRow("select title, description, category_id from products where id = 1").Scan(&p.title, &p.description, &p.categoryId)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
	}
}

func BenchmarkMySqlComplexBind(b *testing.B) {
	db, err := sql.Open("mysql", mySqlConnectionString)
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	stmt, err := db.Prepare("select title, description, category_id from products where id = ?")
	if err != nil {
		log.Fatalf("failed to prepare the statement: %v", err)
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var p product
		err := stmt.QueryRow(1).Scan(&p.title, &p.description, &p.categoryId)
		if err != nil {
			b.Fatalf("Query failed: %v", err)
		}
		//if result != 123 {
		//	b.Fatalf("Expected 123, got %d", result)
		//}
	}
}
