# Rust-IRC Tutorial

This tutorial is designed to walk you through the basic functionality of the
`rust-irc` library. This tutorial will not teach you Rust. If you do not know
Rust, please read through the [Rust Documentation](http://doc.rust-lang.org/).

## Getting Started

First, create a new cargo project. We'll need to add the rust-irc library to our
dependencies:

```
[dependencies.rustirc]
git = "https://github.com/Lancey6/rust-irc"
```

Now let's set up our source file. We'll start with a simple program that
connects to the rust IRC channel. There's a couple of components from the
library that are essential to running a client. Set up your main.rs like this:

```rust
extern crate rustirc;

use rustirc::info::IrcInfo;
use rustirc::client::Client;

fn main() {

}
```

We've imported our Client and IrcInfo structs which are the backbone of our
client. These two components will let us get a simple IRC client up and running,
but it won't have much functionality.

First, we need to give our client some info such as the nick, username, and
realname we want to use and any channels we want to connect to on startup. Add
this line to your `main()` function:

```rust
let info = IrcInfo::gen( "MyNickname", "MyUsername", "MyRealname", vec!["#rust"] );
```

This generates an IrcInfo struct with the nickname "MyNickname", username
"MyUsername", and realname "MyRealname". In addition, when we connect to a
server, our client will automatically try to join the #rust channel.

Having the info is nice, but we want to give this info to a client so we can
connect to IRC. Add this line to your `main()` function:

```rust
let preclient = Client::connect( "irc.mozilla.org", 6667, "", Box::new( info ) );
```

Let's go over these parameters in more detail. 

 * The first parameter is the host to connect to. This can be a master server, a
specific server, an IP address, or anything that allows us to connect to an IRC 
server.
 * The second parameter is the port to connect to. 6667 is the standard IRC 
port.
 * The third parameter is the server password. Since irc.mozilla.org is a public
server, we can leave this blank. If you connect to a password protected server,
this is where you would put the password.
 * The last parameter is the info the client will connect with. We pass in the
info we just generated in the last step.

Notice that we called our new client "preclient". That's because we're not yet 
connected. Once we start our client, we'll get an active client that we can 
operate on. Rust-IRC runs the client in a separate thread so that we don't have 
to wait for the server to send us messages in order to do anything.

Let's start the client now. Add this line to your `main()` function:

```rust
let (rx,mut cnt) = preclient.start_thread( );
```

This line has a lot going on. We start the client thread and connect to the IRC
server. The client then gives us the receiving end of the thread's channel and
another client that's now connected to IRC.

> *NOTE*
> The original client is consumed by the `start_thread` function. From now on,
> any interactions we have with the client will be through the new struct we get
> back from `start_thread`.

Of course, if we run this we'll start the client thread and the program will
immediately terminate, meaning we won't even connect to the server. Let's add
this to our `main()` function so that we can at least connect:

```rust
for msg in rx.iter() { }
```

Now we wait to receive info from the server. This loop will continue to run
until the server closes our connection. It doesn't do anything, but we can add
functionality later.

There's one more thing we should do before our client is "done". We need to
clean up the client after our connection ends. Add one more line to `main()`:

```rust
cnt.stop( );
```

This just frees up any data used by the client and closes the IRC connection.

Your source file should now look something like this:

```rust
extern crate rustirc;

use rustirc::info::IrcInfo;
use rustirc::client::Client;

fn main() {
  let info = IrcInfo::gen( "MyNickname", "MyUsername", "MyRealname", vec!["#rust"] );
  let preclient = Client::connect( "irc.mozilla.org", 6667, "", info );
  let (rx,mut cnt) = preclient.start_thread( );
  
  for msg in rx.iter() { }
  
  cnt.stop( );
}
```

This is a simple client that connects to the #rust IRC channel and waits there
until it loses connection. In the next sections we'll give our client some
actual functionality.