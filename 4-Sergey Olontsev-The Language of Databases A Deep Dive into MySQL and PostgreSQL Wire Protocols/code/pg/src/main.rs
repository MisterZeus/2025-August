use std::ffi::CStr;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use crate::RequestMessage::{Bind, Close, Describe, Execute, Parse, SimpleQuery, Sync, Termination};

#[derive(Debug)]
enum ResponseMessage {
    AuthRequestOK,
    ParameterStatus(String, String),
    ReadyForQuery,
    EmptyQuery,
    SimpleRowDescription,
    SimpleDataRow,
    SimpleCommandCompletion,
    ComplexRowDescription,
    ComplexDataRow1,
    ComplexDataRow2,
    ComplexCommandCompletion,
    ParseCompletion,
    ParameterDescription,
    BindCompletion,
    DataRow,
    CloseCompletion,
}

#[derive(Debug)]
enum RequestMessage {
    SimpleQuery(String),
    Termination,
    Parse,
    Describe,
    Sync,
    Bind,
    Execute,
    Close,
}

impl ResponseMessage {
    fn as_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        match self {
            ResponseMessage::AuthRequestOK => {
                response.extend(0u32.to_be_bytes()); // OK(0), 4 bytes
            }
            ResponseMessage::ParameterStatus(name, value) => {
                response.extend(name.as_bytes());
                response.push(0x00);
                response.extend(value.as_bytes());
                response.push(0x00);
            }
            ResponseMessage::ReadyForQuery => {
                response.push(0x49); // Idle (73)
            }
            ResponseMessage::EmptyQuery => {}
            ResponseMessage::SimpleRowDescription => {
                response.extend(1u16.to_be_bytes()); // field count
                response.extend(b"id\0"); // column name
                response.extend(0u32.to_be_bytes()); // table OID
                response.extend(0u16.to_be_bytes()); // column index
                response.extend(23u32.to_be_bytes()); // type OID
                response.extend(4u16.to_be_bytes()); // column length
                response.extend([0xff, 0xff, 0xff, 0xff]); // type modifier
                response.extend(0u16.to_be_bytes()); // format: text (0)
            }
            ResponseMessage::SimpleDataRow => {
                response.extend(1u16.to_be_bytes()); // field count
                response.extend(3u32.to_be_bytes()); // column length
                response.extend(b"123");
            }
            ResponseMessage::SimpleCommandCompletion => {
                response.extend(b"SELECT 1\0");
            }
            ResponseMessage::ComplexRowDescription => {
                response.extend(4u16.to_be_bytes()); // field count

                response.extend(b"id\0"); // column name
                response.extend([0x00, 0x00, 0x40, 0x01]); // table OID
                response.extend(1u16.to_be_bytes()); // column index
                response.extend(23u32.to_be_bytes()); // type OID
                response.extend(4u16.to_be_bytes()); // column length
                response.extend([0xff, 0xff, 0xff, 0xff]); // type modifier (-1)
                response.extend(0u16.to_be_bytes()); // format: text (0)

                response.extend(b"title\0"); // column name
                response.extend([0x00, 0x00, 0x40, 0x01]); // table OID
                response.extend(3u16.to_be_bytes()); // column index
                response.extend(1043u32.to_be_bytes()); // type OID
                response.extend([0xff, 0xff]); // column length
                response.extend([0x00, 0x00, 0x00, 0x68]); // type modifier (104)
                response.extend(0u16.to_be_bytes()); // format: text (0)

                response.extend(b"description\0"); // column name
                response.extend([0x00, 0x00, 0x40, 0x01]); // table OID
                response.extend(4u16.to_be_bytes()); // column index
                response.extend(25u32.to_be_bytes()); // type OID
                response.extend([0xff, 0xff]); // column length
                response.extend([0xff, 0xff, 0xff, 0xff]); // type modifier (-1)
                response.extend(0u16.to_be_bytes()); // format: text (0)

                response.extend(b"category_id\0"); // column name
                response.extend([0x00, 0x00, 0x40, 0x01]); // table OID
                response.extend(5u16.to_be_bytes()); // column index
                response.extend(21u32.to_be_bytes()); // type OID
                response.extend(2u16.to_be_bytes()); // column length
                response.extend([0xff, 0xff, 0xff, 0xff]); // type modifier (-1)
                response.extend(0u16.to_be_bytes()); // format: text (0)
            }
            ResponseMessage::ComplexDataRow1 => {
                response.extend(4u16.to_be_bytes()); // field count

                response.extend(1u32.to_be_bytes()); // column length
                response.extend(b"1");

                response.extend(6u32.to_be_bytes()); // column length
                response.extend(b"laptop");

                response.extend([0xff, 0xff, 0xff, 0xff]); // column length (-1)

                response.extend(1u32.to_be_bytes()); // column length
                response.extend(b"2");
            }
            ResponseMessage::ComplexDataRow2 => {
                response.extend(4u16.to_be_bytes()); // field count

                response.extend(1u32.to_be_bytes()); // column length
                response.extend(b"2");

                response.extend(5u32.to_be_bytes()); // column length
                response.extend(b"phone");

                response.extend(17u32.to_be_bytes()); // column length
                response.extend(b"Just a phone desc");

                response.extend(5u32.to_be_bytes()); // column length
                response.extend(b"20000");
            }
            ResponseMessage::ComplexCommandCompletion => {
                response.extend(b"SELECT 2\0");
            }
            ResponseMessage::ParseCompletion => {}
            ResponseMessage::ParameterDescription => {
                response.extend([0x00, 0x00]); // parameters
            }
            ResponseMessage::BindCompletion => {}
            ResponseMessage::DataRow => {
                response.extend(1u16.to_be_bytes()); // field count
                response.extend(4u32.to_be_bytes()); // column length
                response.extend(123u32.to_be_bytes());
            }
            ResponseMessage::CloseCompletion => {}
        }

        response
    }

    fn message_type(&self) -> u8 {
        match self {
            ResponseMessage::AuthRequestOK => 0x52, // R
            ResponseMessage::ParameterStatus(_, _) => 0x53, // S
            ResponseMessage::ReadyForQuery => 0x5a, // Z
            ResponseMessage::EmptyQuery => 0x49, // I

            ResponseMessage::SimpleRowDescription => 0x54, // T
            ResponseMessage::SimpleDataRow => 0x44, // D
            ResponseMessage::SimpleCommandCompletion => 0x43, // C

            ResponseMessage::ComplexRowDescription => 0x54, // T
            ResponseMessage::ComplexDataRow1 => 0x44, // D
            ResponseMessage::ComplexDataRow2 => 0x44, // D
            ResponseMessage::ComplexCommandCompletion => 0x43, // C

            ResponseMessage::ParseCompletion => 0x31,
            ResponseMessage::ParameterDescription => 0x74,

            ResponseMessage::BindCompletion => 0x32,
            ResponseMessage::DataRow => 0x44, // D

            ResponseMessage::CloseCompletion => 0x33,
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

fn send_message(stream: &mut TcpStream, msg: ResponseMessage) -> Result<(), std::io::Error> {
    stream.write_all(&[msg.message_type()])?;

    let data = msg.as_bytes();
    let message_len = (data.len() as u32) + 4; // 4 bytes is length itself

    stream.write_all(&message_len.to_be_bytes())?;
    stream.write_all(&data)?;

    stream.flush()?;
    Ok(())
}

fn read_message(stream: &mut TcpStream) -> Result<RequestMessage, std::io::Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    let message_type = u8::from_be_bytes(buf);

    let mut length = [0u8; 4];
    stream.read_exact(&mut length)?;
    let message_len = u32::from_be_bytes(length) as usize - 4; // subtract 4 bytes representing length

    let mut buf = vec![0; message_len];
    stream.read_exact(&mut buf)?;

    match message_type {
        // Simple Query (Q)
        0x51 => {
            buf.pop(); // remove last 0
            Ok(SimpleQuery(String::from_utf8_lossy(buf.as_slice()).to_string()))
        }
        0x58 => Ok(Termination), // X
        0x50 => Ok(Parse), // P
        0x44 => Ok(Describe), // D
        0x53 => Ok(Sync), // S
        0x42 => Ok(Bind), // B
        0x45 => Ok(Execute), // E
        0x43 => Ok(Close), // C
        _ => Err(std::io::ErrorKind::Unsupported.into()),
    }
}

fn read_startup_message(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length)?;
    let message_len = u32::from_be_bytes(length) as usize - 4; // subtract 4 bytes representing length
    // println!("Message length: {}", message_len);

    let mut buf = vec![0; message_len];
    stream.read_exact(&mut buf)?;

    Ok(buf)
}

fn handle_connection(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap_or_else(|_| "unknown".parse().unwrap());
    println!("New connection from: {}", peer_addr);

    let startup_message = read_startup_message(&mut stream);
    // print_message(startup_message.unwrap(), "startup");

    let _ = send_message(&mut stream, ResponseMessage::AuthRequestOK);
    let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);

    loop {
        match read_message(&mut stream) {
            Ok(msg) => match msg {
                SimpleQuery(query) => {
                    match query.as_str() {
                        // ping
                        ";" => {
                                let _ = send_message(&mut stream, ResponseMessage::EmptyQuery);
                                let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                        }
                        "select 123 as id" => {
                            let _ = send_message(&mut stream, ResponseMessage::SimpleRowDescription);
                            let _ = send_message(&mut stream, ResponseMessage::SimpleDataRow);
                            let _ = send_message(&mut stream, ResponseMessage::SimpleCommandCompletion);
                            let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                        }
                        "select id, title, description, category_id from products" => {
                            let _ = send_message(&mut stream, ResponseMessage::ComplexRowDescription);
                            let _ = send_message(&mut stream, ResponseMessage::ComplexDataRow1);
                            let _ = send_message(&mut stream, ResponseMessage::ComplexDataRow2);
                            let _ = send_message(&mut stream, ResponseMessage::ComplexCommandCompletion);
                            let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                        }
                        _ => unimplemented!(),
                    }
                }
                Termination => {
                    break;
                }
                Parse => {
                    if let Ok(desc_msg) = read_message(&mut stream) &&
                        let Ok(sync_msg) = read_message(&mut stream) &&
                        matches!(desc_msg, Describe) && matches!(sync_msg, Sync) {

                        let _ = send_message(&mut stream, ResponseMessage::ParseCompletion);
                        let _ = send_message(&mut stream, ResponseMessage::ParameterDescription);
                        let _ = send_message(&mut stream, ResponseMessage::SimpleRowDescription);
                        let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                    }
                }
                Bind => {
                    if let Ok(exec_msg) = read_message(&mut stream) &&
                        let Ok(sync_msg) = read_message(&mut stream) &&
                        matches!(exec_msg, Execute) && matches!(sync_msg, Sync) {

                        let _ = send_message(&mut stream, ResponseMessage::BindCompletion);
                        let _ = send_message(&mut stream, ResponseMessage::DataRow);
                        let _ = send_message(&mut stream, ResponseMessage::SimpleCommandCompletion);
                        let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                    }
                }
                Close => {
                    if let Ok(sync_msg) = read_message(&mut stream) &&
                        matches!(sync_msg, Sync) {

                        let _ = send_message(&mut stream, ResponseMessage::CloseCompletion);
                        let _ = send_message(&mut stream, ResponseMessage::ReadyForQuery);
                    }
                }
                _ => unimplemented!(),
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    println!("Client {} disconnected", peer_addr);
                } else {
                    println!("Error reading from {}: {}", peer_addr, e);
                }
                break;
            }
        }
    }
}

fn main() {
    let addr = "0.0.0.0:5432";
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