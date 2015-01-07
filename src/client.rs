use config;
use connection;
use info;

/// `Client` is an abstract IRC client
///
/// # Members
///
/// `info` - the client's IrcInfo struct
/// `conn` - the client's connection struct and manager
/// `events` - the client's event dispatcher
pub struct Client <'cl> {
  info      : info::IrcInfo,
  conn      : connection::ServerConnection,
  events    : events::EventDispatcher,
}

impl <'cl> Client <'cl> {
  /// `connect` creates a new IRC client and connects to a server
  ///
  /// # Arguments
  ///
  /// `cfg` - the associated config struct
  ///
  /// # Returns
  ///
  /// A new, connected IRC client
  pub fn connect<'a>( cfg : config::IrcConfig<'a> ) -> Client<'a> {
    let mut clnt = Client {
      info    : info::IrcInfo::new( cfg ),
      conn    : connection::ServerConnection::new( cfg.host, cfg.port ),
      events  : events::EventDispatcher::new( ),
    }
    
    clnt
  }
  
  /// `disconnect` closes the IRC connection
  pub fn disconnect( &mut self ) {
    self.conn.exec( "QUIT" );
    self.info.channels.clear();
    self.conn.close_when_done();
  }
  
  /// `is_connected` gets whether the current IRC connection is active
  pub fn is_connected( &self ) -> bool {
    self.conn.connected
  }
  
  /// `join` connects to a channel
  ///
  /// # Arguments
  ///
  /// `channel` - the channel you want to connect to
  pub fn join( &mut self, channel : &str ) {
    if output::check_conn_exec( 
      self.conn.exec( "JOIN", vec![ channel ] ), 
      "joining channel" 
    ) {
      info.channels.push( channel );
    }
  }
  
  /// `nick` changes the client nickname
  ///
  /// # Arguments
  ///
  /// `nick` - the nickname to change to
  pub fn nick( &mut self, nick : &str ) {
    if output::check_conn_exec(
      self.conn.exec( "NICK", vec![ nick ] ),
      "changing nickname"
    ) {
      info.nick_name = nick;
    }
  }
  
  /// `part` disconnects from a channel
  ///
  /// # Arguments
  ///
  /// `channel` - the channel to part from
  pub fn part( &mut self, channel : &str ) {
    if output::check_conn_exec(
      self.conn.exec( "PART", vec![ channel ] ),
      "parting channel"
    ) {
      info.channels.pop( channel );
    }
  }
  
  /// `run` starts the client and begins acting
  pub fn run( &self ) {
    self.conn.start();
  }
  
  /// `send_action` sends an action message to the target
  ///
  /// # Arguments
  ///
  /// `target` - the channel or user to send the action to
  /// `message` - the action to perform
  pub fn send_action( &self, target : &str, message : &str ) {
    output::check_conn_exec(
      self.conn.exec( "ACTION", vec![ target, ctcp::quote( message ) ] ),
      "sending action"
    );
  }
  
  /// `send_message` sends a message to the target channel or user
  ///
  /// # Arguments
  ///
  /// `target` - the channel or user to send the message to
  /// `message` - the message to say
  pub fn send_message( &self, target : &str, message : &str ) {
    output::check_conn_exec(
      self.conn.exec( "PRIVMSG", vec![ target, ctcp::quote( message ) ] ),
      "sending message"
    );
  }
  
  /// `send_notice` sends a notice to the target channel or user
  ///
  /// # Arguments
  ///
  /// `target` - the channel or user to send the notice to
  /// `message` - the contents of the notice
  pub fn send_notice( &self, target, message ) {
    output::check_conn_exec(
      self.conn.exec( "NOTICE", vec![ target, ctcp::quote( message ) ] ),
      "sending notice"
    );
  }
}