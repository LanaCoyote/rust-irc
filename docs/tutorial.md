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

## Receiving Basic Messages
Having an IRC client that doesn't do anything isn't particularly useful. Let's
make it print every message we receive to the console so that we can participate
in a discussion.

We already have most of the work done in this line here:

```rust
for msg in rx.iter() { }
```

However, the client passes back all the data it receives from the IRC server,
not necessarily the messages from other users that we want to see. We need to do
a bit of work to get only nice messages.

In IRC, messages passed around in a channel are all denoted as being `PRIVMSG`. We
can look at the message code from our data and see if it's a private message,
and if it is, print it to the console!

```rust
for msg in rx.iter() {
  match msg.code.as_slice( ) {
    "PRIVMSG" => println! ( "{}", msg.trailing().unwrap() ),
    _         => (),
  }
}
```

You'll notice that we used the `trailing()` function. This gets the last
parameter from an IRC message, which in the case of a PRIVMSG is the message
itself. Because not all IRC messages will have a trailing parameter, we need to
unwrap it from the `Option` we get.

However, the way we're printing it we just get the message. We don't
know who sent it or where it came from. Let's add some more to our print
statement.

```rust
for msg in rx.iter() {
  match msg.code.as_slice( ) {
    "PRIVMSG" => println! ( "{}: {}", msg.nick().unwrap(), msg.trailing().unwrap() ),
    _         => (),
  }
}
```

The `nick()` function gets the nickname of whoever sent the message. Not all
messages will have a nick, however, so we have to unwrap it if we want the
actual nick, just like with the `trailing()` function.

Now at least we know who said what, but what if we're in multiple channels and
want to know where the messages are coming from? We can look at what channel
the message is sent to by using the `param()` function.

```rust
for msg in rx.iter() {
  match msg.code.as_slice( ) {
    "PRIVMSG" => println! ( "[{}] {}: {}", msg.param( 1 ).unwrap(), msg.nick().unwrap(), msg.trailing().unwrap() ),
    _         => (),
  }
}
```

The first parameter in a PRIVMSG is the destination, so we now know where the
message was sent to. Notice that like the other functions we've used, we needed
to unwrap it. Again, there's no guarantee that a message will have a first
parameter, so we have to get it ourselves.

Now we get output from the IRC server! Here's a sample of what we see in the
console:

```
[#rust] Lancey: Hello!
[#rust] Lancey: What's up?
```

## Handling Other Messages
You'll notice that even though we now receive messages from the channel, we have
no idea what else is happening. For example, we don't know who's entering or
exiting. Let's make our IRC client a bit more sophisticated.

First let's add a message that shows when someone joins the channel. The code
for joining a channel is `JOIN`, so let's add it now:

```rust
for msg in rx.iter() {
  match msg.code.as_slice( ) {
    "PRIVMSG" => println! ( "[{}] {}: {}", msg.param( 1 ).unwrap(), msg.nick().unwrap(), msg.trailing().unwrap() ),
    "JOIN"    => println! ( "[{}] {} joined the channel", msg.param( 1 ).unwrap(), msg.nick().unwrap() ),
    _         => (),
  }
}
```

The parameters of a `JOIN` message are very similar to a `PRIVMSG` message,
except we don't have a trailing parameter.

Now let's add a message for when someone leaves. Leaving a channel is done
through a `PART` message.

```rust
for msg in rx.iter() {
  match msg.code.as_slice( ) {
    "PRIVMSG" => println! ( "[{}] {}: {}", msg.param( 1 ).unwrap(), msg.nick().unwrap(), msg.trailing().unwrap() ),
    "JOIN"    => println! ( "[{}] {} joined the channel", msg.param( 1 ).unwrap(), msg.nick().unwrap() ),
    "PART"    => println! ( "[{}] {} left the channel", msg.param( 1 ).unwrap(), msg.nick().unwrap() ),
    _         => (),
  }
}
```

`PART` messages do have a trailing parameter that gives a parting message. See
if you can figure out how to add it to what we have above so that we see those
parting messages.

*to be continued*