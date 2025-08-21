package main

import (
	"database/sql"
	"fmt"
	"log"

	_ "github.com/lib/pq"
)

type Product struct {
	id          int
	title       string
	description sql.NullString
	categoryId  sql.NullInt16
}

var id int

func main() {
	db, err := sql.Open("postgres", "postgres://postgres:let-me-in@localhost:5432/protocols?sslmode=disable")
	if err != nil {
		log.Fatalf("failed to connect without compression: %v", err)
	}
	defer db.Close()

	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)

	// 1. Ping (test connection)
	//if err = db.Ping(); err != nil {
	//	log.Fatalf("failed to ping without compression: %v", err)
	//}

	// 2. Select 1 column, 1 row
	//err = db.QueryRow("select 123 as id").Scan(&id)
	//if err != nil {
	//	log.Fatalf("Query failed: %v", err)
	//}
	//fmt.Println("Result:", id)

	// 3. More complex query
	//rows, err := db.Query("select id, title, description, category_id from products")
	//var products []Product
	//if err != nil {
	//	log.Fatalf("Query failed: %v", err)
	//}
	//for rows.Next() {
	//	var p Product
	//
	//	err := rows.Scan(&p.id, &p.title, &p.description, &p.categoryId)
	//	if err != nil {
	//		log.Fatalf("Scan failed: %v", err)
	//	}
	//
	//	products = append(products, p)
	//}
	//fmt.Println(products)

	// 4. Prepared statement
	stmt, err := db.Prepare("select 123 as id")
	if err != nil {
		log.Fatalf("failed to prepare the statement: %v", err)
	}
	err = stmt.QueryRow().Scan(&id)
	if err != nil {
		log.Fatalf("Query failed: %v", err)
	}
	fmt.Println("Result:", id)

	fmt.Println("Done.")
}
