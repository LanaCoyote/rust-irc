# rust-irc
Library for managing IRC connections in rustlang. Last updated 1/15/2015.

## Features

 - Asynchronous connection and i/o
 - Automatically manages pings and server registration
 - CTCP support
 - Structured message handling

## Example

```rust
let info      = rustirc::info::IrcInfo( "MyIrcTest", "MyIrcTest", "Testing rust-irc", vec!["#rust"] );
let client    = rustirc::client::Client::connect( "irc.mozilla.org", 6667, "", info );
let (rx,_)    = client.start_thread( );

for msg in rx.iter( ) {
  println! ( " > {}", msg.raw );
}

client.close( );
```
