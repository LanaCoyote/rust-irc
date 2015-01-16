# rust-irc
Library for managing IRC connections in rustlang. Last updated 1/15/2015.

## Features

 - Asynchronous connection and i/o
 - Automatically manages pings and server registration
 - CTCP support
 - Structured message handling

## To-do

 - Channel user lists (hard to sync between threads)
 - Numeric code constants
 - And more

## Example

```rust
let info      = rustirc::info::IrcInfo::gen( "MyIrcTest", "MyIrcTest", "Testing rust-irc", vec!["#rust"] );
let preclient = rustirc::client::Client::connect( "irc.mozilla.org", 6667, "", Box::new( info ) );
let (rx,cnt)  = preclient.start_thread( );

for msg in rx.iter( ) {
  println! ( " > {}", msg.raw );
}

cnt.close( );
```
