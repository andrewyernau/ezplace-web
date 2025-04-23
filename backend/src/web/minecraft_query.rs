// minecraft_query.rs 
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use byteorder::{BigEndian, WriteBytesExt};

use crate::Error;
use crate::Result;
use crate::models::{Players, ServerStats};

pub fn query_minecraft_server(host: &str, port: u16) -> Result<ServerStats> {
    let addr = format!("{}:{}", host, port)
        .to_socket_addrs()
        .map_err(|e| Error::ServerError(format!("Socket error: {}", e)))?
        .next()
        .ok_or(Error::ServerError("Could not resolve host".to_string()))?;

    // Connect
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5))
        .map_err(|e| Error::ServerError(format!("Failed to connect: {}", e)))?;
    
    stream.set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| Error::ServerError(format!("Failed to set read timeout: {}", e)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| Error::ServerError(format!("Failed to set write timeout: {}", e)))?;
    send_handshake(&mut stream, host, port)?;
    send_status_request(&mut stream)?;
    let response = read_status_response(&mut stream)?;
    //println!("{:#?}", response);
    parse_status_response(&response)
}

fn send_handshake(stream: &mut TcpStream, host: &str, port: u16) -> Result<()> {
    let protocol_version = 47; 
    let next_state = 1;
    
    let mut packet_data = Vec::new();
    
    // Packet ID (0x00 for handshake)
    packet_data.write_varint(0x00)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    
    // Protocol number
    packet_data.write_varint(protocol_version)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    
    // Hostname
    packet_data.write_string(host)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    
    // port
    packet_data.write_u16::<BigEndian>(port)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    
    // Next state
    packet_data.write_varint(next_state)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;

    send_packet(stream, &packet_data)
}

fn send_status_request(stream: &mut TcpStream) -> Result<()> {
    let mut packet_data = Vec::new();

    packet_data.write_varint(0x00)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    
    send_packet(stream, &packet_data)
}

fn read_status_response(stream: &mut TcpStream) -> Result<String> {
    let _packet_length = read_varint(stream)?;
    
    let packet_id = read_varint(stream)?;
    
    if packet_id != 0x00 {
        return Err(Error::ServerError(format!("Unexpected packet ID: {}", packet_id)));
    }
    
    let json_length = read_varint(stream)?;
    
    let mut json_data = vec![0u8; json_length as usize];
    stream.read_exact(&mut json_data)
        .map_err(|e| Error::ServerError(format!("Failed to read JSON data: {}", e)))?;
    
    String::from_utf8(json_data)
        .map_err(|e| Error::ServerError(format!("Invalid UTF-8: {}", e)))
}

fn parse_status_response(response: &str) -> Result<ServerStats> {
    let json: serde_json::Value = serde_json::from_str(response)
        .map_err(|e| Error::ServerError(format!("Failed to parse JSON: {}", e)))?;
    
    // Get all data
    let protocol_name: &str = json
        .get("version")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    
    let players_online: u32 = json
        .get("players")
        .and_then(|p| p.get("online"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    
    let players_max: u32 = json
        .get("players")
        .and_then(|p| p.get("max"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    
    Ok(ServerStats {
        online: true,
        protocol_name: extract_version(protocol_name).to_owned(),
        players: Players {
            online: players_online,
            max: players_max,
        },
    })
}

fn extract_version(full_version: &str) -> &str {
    full_version.split('-').nth(1).unwrap_or("Unknown")
}

fn send_packet(stream: &mut TcpStream, data: &[u8]) -> Result<()> {
    let mut packet = Vec::new();
    packet.write_varint(data.len() as i32)
        .map_err(|e| Error::ServerError(format!("Protocol error: {}", e)))?;
    packet.extend_from_slice(data);
    
    stream.write_all(&packet)
        .map_err(|e| Error::ServerError(format!("Failed to send packet: {}", e)))?;
    stream.flush()
        .map_err(|e| Error::ServerError(format!("Failed to flush stream: {}", e)))?;
    
    Ok(())
}

trait WriteMinecraftExt: WriteBytesExt {
    fn write_varint(&mut self, val: i32) -> std::io::Result<()> {
        let mut value = val as u32;
        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b10000000;
            }
            self.write_u8(temp)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
    }
    
    fn write_string(&mut self, s: &str) -> std::io::Result<()> {
        self.write_varint(s.len() as i32)?;
        self.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl<W: WriteBytesExt> WriteMinecraftExt for W {}

fn read_varint(stream: &mut TcpStream) -> Result<i32> {
    let mut result = 0;
    let mut num_read = 0;
    
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte)
            .map_err(|e| Error::ServerError(format!("Failed to read VarInt: {}", e)))?;
        
        let value = byte[0] & 0b01111111;
        result |= (value as i32) << (7 * num_read);
        
        num_read += 1;
        if num_read > 5 {
            return Err(Error::ServerError("VarInt too big".to_string()));
        }
        
        if byte[0] & 0b10000000 == 0 {
            break;
        }
    }
    
    Ok(result)
}