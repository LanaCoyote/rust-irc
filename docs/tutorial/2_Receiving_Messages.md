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