mod settings;

extern crate directories;
extern crate config;
extern crate serde;
extern crate imap;
extern crate imap_proto;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::{Utf8Error, from_utf8};

use imap::{Client, Connection, Error::*, types::Fetch, Error};
use native_tls::TlsStream;
use settings::Credentials;

use crate::settings::{Settings, Server};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use imap_proto::types::Address;

enum ConnectionError {
    // Error in the configuration
    ConfigError(String),
    // Error from Imap
    ImapError(imap::Error),
    // Utf8 encodnig error
    EncodingError(Utf8Error)
}

impl From<imap::Error> for ConnectionError {
    fn from(e: Error) -> Self { ConnectionError::ImapError(e) }
}

impl From<(imap::Error, Client<TlsStream<TcpStream>>)> for ConnectionError {
    fn from(e: (Error, Client<TlsStream<TcpStream>>)) -> Self { ConnectionError::ImapError(e.0) }
}

fn print_addresses(head: &str, addresses: &Vec<Address>) {
    print!("{:}", head);
    for address in addresses {
        print!("(");
        let decode_to_str = |bytes| from_utf8(bytes).ok();
        if let Some(name) = address.name.and_then(decode_to_str) { print!("\"{:}\"", name) } else { print!("NIL") }
        print!(" ");
        if let Some(adl) = address.adl.and_then(decode_to_str) { print!("\"{:}\"", adl) } else { print!("NIL") }
        print!(" ");
        if let Some(mailbox) = address.mailbox.and_then(decode_to_str) { print!("\"{:}\"", mailbox) } else { print!("NIL") }
        print!(" ");
        if let Some(host) = address.host.and_then(decode_to_str) { print!("\"{:}\"", host) } else { print!("NIL") }
        print!("), ");
    }
    println!();
}

fn list_inbox(server: &Server) -> Result<(), ConnectionError> {
    let credentials = server.credentials();
    let name = server.name();

    let server = server.imap();
    let domain = server.host();
    let port = server.port();
    let client = server.tls()
        .ok_or_else(|| ConnectionError::ConfigError(format!("No TLS configured for '{:}'", name)))
        .and_then(|_| {
            let tls = native_tls::TlsConnector::builder().build().unwrap();        
            imap::connect((domain, port), domain, &tls).map_err(ConnectionError::from)
        })?;


    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = match credentials {
        Credentials::UsernameAndPassword { username, password } => client.login(username, password)?,
        Credentials::None => return Err(ConnectionError::ConfigError(format!("No username and password configured for '{:?}'", name))),
    };

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;
    

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = imap_session.fetch("1:100", "ALL")?;
    for message in messages.iter() {
        println!("---");
        if let Some(envelope) = message.envelope() {
            if let Some(from) = &envelope.from {
                print_addresses("From: ", from);
            }
            if let Some(to) = &envelope.to {
                print_addresses("To: ", to);
            }
            if let Some(cc) = &envelope.cc {
                print_addresses("Cc: ", cc);
            }
            if let Some(bcc) = &envelope.bcc {
                print_addresses("Bcc: ", bcc);
            }
            if let Some(date) = &envelope.date.and_then(|v| from_utf8(v).ok() ) {
                println!("Date: {:}", *date);
            }
            if let Some(subject) = &envelope.subject.and_then(|v| from_utf8(v).ok() ) {
                println!("Subject: {:}", *subject);
            }
        }
    }

    // be nice to the server and log out
    imap_session.logout()?;

    Ok(())
}

fn main() {
    match Settings::load() {
        Err(err) => {
            if let Some(internal_err) = Settings::print_default().err() {
                eprintln!("ERR: {:?}", internal_err);
            }
            exit_with_message(1, err.to_string())
        },
        Ok(settings) => {
            for server in settings.servers() {
                match list_inbox(&server) {
                    Ok(_) => println!("---\nDone."),
                    Err(ConnectionError::ImapError(No(msg))) => exit_with_message(1, format!("Invalid 0")),
                    Err(ConnectionError::ImapError(e)) => eprintln!("{:?}", &e),
                    Err(ConnectionError::ConfigError(e)) => eprintln!("CONFIG: {:?}", e),
                    Err(ConnectionError::EncodingError(e)) => eprintln!("Encoding: {:?}", e),
                }
            }
        }
    }
}

fn exit_with_message(exit_status: i32, message: String) {
    eprintln!("ERROR: {:?}", message);
    std::process::exit(exit_status);
}
