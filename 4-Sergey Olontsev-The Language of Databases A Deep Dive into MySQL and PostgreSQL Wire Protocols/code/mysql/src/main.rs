use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const MAX_PACKET_SIZE: u32 = 16_777_215; // 2 ** 24 - 1

#[derive(Debug)]
enum Packet {
    Greeting,
    AuthSuccess,
    OK,
    ColumnCount(u8),
    SimpleFieldPacket,
    Eof,
    SimpleRowPacket,
    IdFieldPacket,
    TitleFieldPacket,
    DescriptionFieldPacket,
    CategoryIdFieldPacket,
    ComplexEof,
    ComplexRow1Packet,
    ComplexRow2Packet,
    PrepareOk,
    PreparedRowPacket,
}

impl Packet {
    fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        match self {
            Packet::Greeting => {
                response.push(0x0A); // 10, protocol version number

                response.extend(b"9.4.0\0"); // server version

                response.extend([0x01, 0x00, 0x00, 0x00]); // thread ID

                response.extend(b"abcdabcd\0"); // salt (part 1)

                // server capabilities
                // .... .... .... ...1 = Long Password: Set
                // .... .... .... ..1. = Found Rows: Set
                // .... .... .... .1.. = Long Column Flags: Set
                // .... .... .... 1... = Connect With Database: Set
                // .... .... ...1 .... = Don't Allow database.table.column: Set
                // .... .... ..1. .... = Can use compression protocol: Set
                // .... .... .1.. .... = ODBC Client: Set
                // .... .... 1... .... = Can Use LOAD DATA LOCAL: Set
                // .... ...1 .... .... = Ignore Spaces before '(': Set
                // .... ..1. .... .... = Speaks 4.1 protocol (new flag): Set
                // .... .1.. .... .... = Interactive Client: Set
                // .... 1... .... .... = Switch to SSL after handshake: Set
                // ...1 .... .... .... = Ignore sigpipes: Set
                // ..1. .... .... .... = Knows about transactions: Set
                // .1.. .... .... .... = Speaks 4.1 protocol (old flag): Set
                // 1... .... .... .... = Can do 4.1 authentication: Set
                response.extend([0xFF, 0xFF]);

                // utf8mb4 COLLATE utf8mb4_0900_ai_ci (255)
                response.push(0xFF);

                // server status 0x0002
                // .... .... .... ...0 = In transaction: Not set
                // .... .... .... ..1. = AUTO_COMMIT: Set
                // .... .... .... .0.. = Multi query / Unused: Not set
                // .... .... .... 0... = More results: Not set
                // .... .... ...0 .... = Bad index used: Not set
                // .... .... ..0. .... = No index used: Not set
                // .... .... .0.. .... = Cursor exists: Not set
                // .... .... 0... .... = Last row sent: Not set
                // .... ...0 .... .... = Database dropped: Not set
                // .... ..0. .... .... = No backslash escapes: Not set
                // .... .0.. .... .... = Metadata changed: Not set
                // .... 0... .... .... = Query was slow: Not set
                // ...0 .... .... .... = PS Out Params: Not set
                // ..0. .... .... .... = In Trans Readonly: Not set
                // .0.. .... .... .... = Session state changed: Not set
                response.extend([0x02, 0x00]);

                // extended server capabilities
                response.extend([0xFF, 0xDF]);

                // authentication plugin length
                response.push(0x15);

                // unused, 10 bytes
                let unused = [0; 10];
                response.extend(unused);

                response.extend(b"abcdabcdabcd\0"); // salt (part 2)

                // authentication plugin
                response.extend(b"caching_sha2_password\0");
            }
            Packet::AuthSuccess => {
                response.extend([0x01, 0x03]); // SHA2 Auth State: fast_auth_success
            }
            Packet::OK => {
                response.push(0x00); // OK
                response.push(0x00); // affected_rows
                response.push(0x00); // last_insert_id

                response.extend([0x02, 0x00]); // status_flags

                response.extend([0x00, 0x00]); // warnings
            }
            Packet::ColumnCount(c) => {
                response.push(*c);
            }
            Packet::Eof => {
                response.push(0xfe);
                response.extend([0x00, 0x00]); // warnings
                response.extend([0x02, 0x00]); // status_flags
            }
            Packet::SimpleFieldPacket => {
                // catalog
                // https://dev.mysql.com/doc/refman/9.4/en/information-schema-schemata-table.html
                response.push(3);
                response.extend(b"def");

                response.push(0x00); // database
                response.push(0x00); // table
                response.push(0x00); // original table

                // name
                response.push(2);
                response.extend(b"id");

                response.push(0x00); // original name

                response.push(0x0c); // length of fixed length fields, 0x0c (12)

                response.extend([0x3f, 0x00]); // charset number, 63, binary COLLATE

                response.extend([0x04, 0x00, 0x00, 0x00]); // length

                response.push(0x08); // FIELD_TYPE_LONGLONG

                // .... .... .... ...1 = Not null: Set
                // .... .... .... ..0. = Primary key: Not set
                // .... .... .... .0.. = Unique key: Not set
                // .... .... .... 0... = Multiple key: Not set
                // .... .... ...0 .... = Blob: Not set
                // .... .... ..0. .... = Unsigned: Not set
                // .... .... .0.. .... = Zero fill: Not set
                // .... ...0 .... .... = Enum: Not set
                // .... ..0. .... .... = Auto increment: Not set
                // .... .0.. .... .... = Timestamp: Not set
                // .... 0... .... .... = Set: Not set
                response.extend([0x81, 0x00]); // flags

                response.push(0x00); // decimals

                response.extend([0x00, 0x00]); // reserved
            }
            Packet::SimpleRowPacket => {
                response.push(3);
                response.extend(b"123");
            }
            Packet::PreparedRowPacket => {
                response.push(0x00); // OK
                response.push(0x00); // row null buffer
                response.extend(123u64.to_le_bytes());
            }
            Packet::IdFieldPacket => {
                // catalog
                // https://dev.mysql.com/doc/refman/9.4/en/information-schema-schemata-table.html
                response.push(3);
                response.extend(b"def");

                // database
                response.push(0x09);
                response.extend(b"protocols");
                // table
                response.push(0x08);
                response.extend(b"products");
                // original table
                response.push(0x08);
                response.extend(b"products");

                // name
                response.push(2);
                response.extend(b"id");

                // original name
                response.push(2);
                response.extend(b"id");

                response.push(0x0c); // length of fixed length fields, 0x0c (12)

                response.extend([0x3f, 0x00]); // charset number, 63, binary COLLATE

                response.extend([0x04, 0x00, 0x00, 0x00]); // length

                response.push(0x03); // FIELD_TYPE_LONG

                // flags
                // .... .... .... ...1 = Not null: Set
                // .... .... .... ..1. = Primary key: Not set
                // .... .... .... .0.. = Unique key: Not set
                // .... .... .... 0... = Multiple key: Not set
                // .... .... ...0 .... = Blob: Not set
                // .... .... ..0. .... = Unsigned: Not set
                // .... .... .0.. .... = Zero fill: Not set
                // .... ...0 .... .... = Enum: Not set
                // .... ..0. .... .... = Auto increment: Not set
                // .... .0.. .... .... = Timestamp: Not set
                // .... 0... .... .... = Set: Not set
                response.extend([0x03, 0x50]);

                response.push(0x00); // decimals

                response.extend([0x00, 0x00]); // reserved
            }
            Packet::TitleFieldPacket => {
                // catalog
                // https://dev.mysql.com/doc/refman/9.4/en/information-schema-schemata-table.html
                response.push(3);
                response.extend(b"def");

                // database
                response.push(0x09);
                response.extend(b"protocols");
                // table
                response.push(0x08);
                response.extend(b"products");
                // original table
                response.push(0x08);
                response.extend(b"products");

                // name
                response.push(5);
                response.extend(b"title");

                // original name
                response.push(5);
                response.extend(b"title");

                response.push(0x0c); // length of fixed length fields, 0x0c (12)

                response.extend([0x2d, 0x00]); // charset number, 45, utf8mb4 COLLATE utf8mb4_general_ci

                response.extend([0x90, 0x01, 0x00, 0x00]); // length, 400

                response.push(0xfd); // FIELD_TYPE_VAR_STRING

                // flags
                // .... .... .... ...1 = Not null: Set
                // .... .... .... ..0. = Primary key: Not set
                // .... .... .... .1.. = Unique key: Set
                // .... .... .... 0... = Multiple key: Not set
                // .... .... ...0 .... = Blob: Not set
                // .... .... ..0. .... = Unsigned: Not set
                // .... .... .0.. .... = Zero fill: Not set
                // .... ...0 .... .... = Enum: Not set
                // .... ..0. .... .... = Auto increment: Not set
                // .... .0.. .... .... = Timestamp: Not set
                // .... 0... .... .... = Set: Not set
                response.extend([0x05, 0x50]);

                response.push(0x00); // decimals

                response.extend([0x00, 0x00]); // reserved
            }
            Packet::DescriptionFieldPacket => {
                // catalog
                // https://dev.mysql.com/doc/refman/9.4/en/information-schema-schemata-table.html
                response.push(3);
                response.extend(b"def");

                // database
                response.push(0x09);
                response.extend(b"protocols");
                // table
                response.push(0x08);
                response.extend(b"products");
                // original table
                response.push(0x08);
                response.extend(b"products");

                // name
                response.push(0x0b);
                response.extend(b"description");

                // original name
                response.push(0x0b);
                response.extend(b"description");

                response.push(0x0c); // length of fixed length fields, 0x0c (12)

                response.extend([0x2d, 0x00]); // charset number, 45, utf8mb4 COLLATE utf8mb4_general_ci

                response.extend([0xff, 0xff, 0xff, 0xff]); // length

                response.push(0xfc); // FIELD_TYPE_BLOB

                // flags
                // .... .... .... ...0 = Not null: Not set
                // .... .... .... ..0. = Primary key: Not set
                // .... .... .... .0.. = Unique key: Not set
                // .... .... .... 0... = Multiple key: Not set
                // .... .... ...1 .... = Blob: Set
                // .... .... ..0. .... = Unsigned: Not set
                // .... .... .0.. .... = Zero fill: Not set
                // .... ...0 .... .... = Enum: Not set
                // .... ..0. .... .... = Auto increment: Not set
                // .... .0.. .... .... = Timestamp: Not set
                // .... 0... .... .... = Set: Not set
                response.extend([0x10, 0x00]);

                response.push(0x00); // decimals

                response.extend([0x00, 0x00]); // reserved
            }
            Packet::CategoryIdFieldPacket => {
                // catalog
                // https://dev.mysql.com/doc/refman/9.4/en/information-schema-schemata-table.html
                response.push(3);
                response.extend(b"def");

                // database
                response.push(0x09);
                response.extend(b"protocols");
                // table
                response.push(0x08);
                response.extend(b"products");
                // original table
                response.push(0x08);
                response.extend(b"products");

                // name
                response.push(0x0b);
                response.extend(b"category_id");

                // original name
                response.push(0x0b);
                response.extend(b"category_id");

                response.push(0x0c); // length of fixed length fields, 0x0c (12)

                response.extend([0x3f, 0x00]); // charset number, 63, binary COLLATE

                response.extend([0x06, 0x00, 0x00, 0x0]); // length, 6

                response.push(0x02); // FIELD_TYPE_SHORT

                // flags
                // .... .... .... ...0 = Not null: Not set
                // .... .... .... ..0. = Primary key: Not set
                // .... .... .... .0.. = Unique key: Not set
                // .... .... .... 0... = Multiple key: Not set
                // .... .... ...0 .... = Blob: Not set
                // .... .... ..0. .... = Unsigned: Not set
                // .... .... .0.. .... = Zero fill: Not set
                // .... ...0 .... .... = Enum: Not set
                // .... ..0. .... .... = Auto increment: Not set
                // .... .0.. .... .... = Timestamp: Not set
                // .... 0... .... .... = Set: Not set
                response.extend([0x00, 0x00]);

                response.push(0x00); // decimals

                response.extend([0x00, 0x00]); // reserved
            }
            Packet::ComplexEof => {
                response.push(0xfe);
                response.extend([0x00, 0x00]); // warnings

                // status_flags
                // .... .... .... ..1. = AUTO_COMMIT: Set
                // .... .... ..1. .... = No index used: Set
                response.extend([0x22, 0x00]);
            }
            Packet::ComplexRow1Packet => {
                // 1
                response.push(1);
                response.extend(b"1");

                // laptop
                response.push(6);
                response.extend(b"laptop");

                // null
                response.push(0xfb);

                // 2
                response.push(1);
                response.extend(b"2");
            }
            Packet::ComplexRow2Packet => {
                // 1
                response.push(1);
                response.extend(b"2");

                // laptop
                response.push(5);
                response.extend(b"phone");

                // null
                response.push(0x11);
                response.extend(b"Just a phone desc");

                // 2
                response.push(5);
                response.extend(b"20000");
            }
            Packet::PrepareOk => {
                response.push(0x00); // OK
                response.extend(1u32.to_le_bytes()); // statement id
                response.extend(1u16.to_le_bytes()); // number of fields
                response.extend(0u16.to_le_bytes()); // number of parameters
                response.push(0x00);
                response.extend(0u16.to_le_bytes()); // warnings
            }
        }

        response
    }
}

#[derive(Debug)]
enum Command {
    Ping,
    Quit,
    Query(String),
    PrepareStmt(String),
    CloseStmt(u32),
    ExecuteStmt(u32, u8, u32), // stmt_id, flags, iterations
}

impl Command {
    fn parse(data: &[u8]) -> Command {
        let command = data[0];

        match command {
            14 => Command::Ping,
            1 => Command::Quit,
            3 => Command::Query(String::from_utf8_lossy(&data[1..]).to_string()),
            22 => Command::PrepareStmt(String::from_utf8_lossy(&data[1..]).to_string()),
            25 => {
                let bytes: [u8; 4] = data[1..5].try_into().expect("slice with incorrect length");
                Command::CloseStmt(u32::from_le_bytes(bytes))
            }
            23 => {
                let bytes: [u8; 4] = data[1..5].try_into().expect("slice with incorrect length");
                let stmt_id = u32::from_le_bytes(bytes);
                let flags = u8::from(data[5]);
                let bytes: [u8; 4] = data[6..10].try_into().expect("slice with incorrect length");
                let iterations = u32::from_le_bytes(bytes);

                Command::ExecuteStmt(stmt_id, flags, iterations)
            }
            _ => unimplemented!(),
        }
    }
}

fn print_message(data: impl AsRef<[u8]>, title: &str) {
     let x = data.as_ref().iter()
         .map(|b| format!("{:02x}", b))
         .collect::<Vec<String>>()
         .join(" ");
    println!("{title}: {x}");
}

fn send_packet(stream: &mut TcpStream, data: &[u8], packet_num: u8) -> Result<(), std::io::Error> {
    // Header: length (3 bytes, little-endian) followed by a packet number
    let packet_len = data.len() as u32;
    if packet_len >= MAX_PACKET_SIZE {
        return Err(std::io::ErrorKind::Unsupported.into());
    }
    let packet_len_bytes = packet_len.to_le_bytes();
    let header = [packet_len_bytes[0], packet_len_bytes[1], packet_len_bytes[2], packet_num];

    stream.write_all(&header)?;
    stream.write_all(data)?;
    stream.flush()?;
    Ok(())
}

fn read_packet(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header)?;

    let packet_num = header[3];
    // let packet_len = header[0] as usize + ((header[1] as usize) << 8) + ((header[2] as usize) << 16);
    let packet_len = u32::from_le_bytes([header[0], header[1], header[2], 0]) as usize;

    let mut buffer = vec![0; packet_len];
    stream.read_exact(&mut buffer)?;

    Ok(buffer)
}

fn handle_connection(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
    println!("Connection established with {}", peer_addr);

    // Authentication
    // {
    //     // Send binary greeting message
    //     let greeting = &Packet::Greeting.as_bytes();
    //     if let Err(e) = send_packet(&mut stream, greeting, 0) {
    //         return;
    //     }
    //
    //     // auth packet
    //     let _ = read_packet(&mut stream);
    //
    //     // auth success packet
    //     if let Err(_) = send_packet(&mut stream, &Packet::AuthSuccess.as_bytes(), 2) {
    //         return;
    //     }
    //
    //     // ok packet
    //     if let Err(_) = send_packet(&mut stream, &Packet::OK.as_bytes(), 3) {
    //         return;
    //     }
    // }

    // Command State
    // loop {
    //     match read_packet(&mut stream) {
    //         Ok(data) => {
    //             let command = Command::parse(data.as_slice());
    //
    //             match command {
    //                 Command::Ping => {
    //                     // ok packet
    //                     let _ = send_packet(&mut stream, &Packet::OK.as_bytes(), 1);
    //                 }
    //                 Command::CloseStmt(_) => {
    //                     continue
    //                 }
    //                 Command::Quit => {
    //                     return;
    //                 }
    //                 Command::PrepareStmt(query) => {
    //                     match query.as_str() {
    //                         "select 123 as id" => {
    //                             let _ = send_packet(&mut stream, &Packet::PrepareOk.as_bytes(), 1);
    //                             let _ = send_packet(&mut stream, &Packet::SimpleFieldPacket.as_bytes(), 2);
    //                             let _ = send_packet(&mut stream, &Packet::Eof.as_bytes(), 3);
    //                         }
    //                         _ => {
    //                             println!("Not supported: {}", query);
    //                             return;
    //                         }
    //                     }
    //                 }
    //                 Command::ExecuteStmt(stmt_id, _, _) => {
    //                     match stmt_id {
    //                         1 => {
    //                             let _ = send_packet(&mut stream, &Packet::ColumnCount(1).as_bytes(), 1);
    //                             let _ = send_packet(&mut stream, &Packet::SimpleFieldPacket.as_bytes(), 2);
    //                             let _ = send_packet(&mut stream, &Packet::Eof.as_bytes(), 3);
    //                             let _ = send_packet(&mut stream, &Packet::PreparedRowPacket.as_bytes(), 4);
    //                             let _ = send_packet(&mut stream, &Packet::Eof.as_bytes(), 5);
    //
    //                         }
    //                         _ => {
    //                             println!("Not supported statement id: {}", stmt_id);
    //                             return;
    //                         }
    //                     }
    //                 }
    //                 Command::Query(query) => {
    //                     match query.as_str() {
    //                         "select 123 as id" => {
    //                             let _ = send_packet(&mut stream, &Packet::ColumnCount(1).as_bytes(), 1);
    //                             let _ = send_packet(&mut stream, &Packet::SimpleFieldPacket.as_bytes(), 2);
    //                             let _ = send_packet(&mut stream, &Packet::Eof.as_bytes(), 3);
    //                             let _ = send_packet(&mut stream, &Packet::SimpleRowPacket.as_bytes(), 4);
    //                             let _ = send_packet(&mut stream, &Packet::Eof.as_bytes(), 5);
    //                         }
    //                         "select id, title, description, category_id from products" => {
    //                             let _ = send_packet(&mut stream, &Packet::ColumnCount(4).as_bytes(), 1);
    //                             let _ = send_packet(&mut stream, &Packet::IdFieldPacket.as_bytes(), 2);
    //                             let _ = send_packet(&mut stream, &Packet::TitleFieldPacket.as_bytes(), 3);
    //                             let _ = send_packet(&mut stream, &Packet::DescriptionFieldPacket.as_bytes(), 4);
    //                             let _ = send_packet(&mut stream, &Packet::CategoryIdFieldPacket.as_bytes(), 5);
    //                             let _ = send_packet(&mut stream, &Packet::ComplexEof.as_bytes(), 6);
    //                             let _ = send_packet(&mut stream, &Packet::ComplexRow1Packet.as_bytes(), 7);
    //                             let _ = send_packet(&mut stream, &Packet::ComplexRow2Packet.as_bytes(), 8);
    //                             let _ = send_packet(&mut stream, &Packet::ComplexEof.as_bytes(), 9);
    //
    //                         }
    //                         _ => {
    //                             println!("Not supported: {}", query);
    //                             return;
    //                         }
    //                     }
    //                 }
    //             }
    //
    //         }
    //         Err(e) => {
    //             if e.kind() == std::io::ErrorKind::UnexpectedEof {
    //                 println!("Client {} disconnected", peer_addr);
    //             } else {
    //                 println!("Error reading from {}: {}", peer_addr, e);
    //             }
    //             break;
    //         }
    //     }
    // }
}

fn main() {
    let addr = "0.0.0.0:3306";
    let listener = TcpListener::bind(addr).expect("failed to bind to address");
    println!("Server listening on {addr}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}