// use modules
use std::collections;
use std::str;

use message;
use utils::debug;

/// `IrcInfo` contains general client information, including the current channel
/// list
///
/// # Members
///
/// * `nick_name` - client nick on the server
/// * `user_name` - username of the client
/// * `real_name` - realname of the client
/// * `channels` - list of channels the client is currently in
pub struct IrcInfo {
  pub nick_name : String,
  pub user_name : String,
  pub real_name : String,

  pub channels  : Vec < String >,
  names         : collections::HashMap < String, Vec < String > >,
  prep_names    : Vec < String >,
}

impl Clone for IrcInfo {
  fn clone( &self ) -> IrcInfo {
    IrcInfo {
      nick_name : self.nick_name.clone( ),
      user_name : self.user_name.clone( ),
      real_name : self.real_name.clone( ),
      channels  : self.channels.clone( ),
      names     : self.names.clone( ),
      prep_names: Vec::new( ),
    }
  }
}

impl IrcInfo {
  /// `gen` generates an IrcInfo struct and returns it
  ///
  /// # Arguments
  ///
  /// `nick` - nickname to set on client
  /// `user` - username to set on client
  /// `real` - realname to set on client
  /// `chans` - vector of channel names to join on connect
  ///
  /// # Returns
  ///
  /// A new IrcInfo struct
  pub fn gen( nick : &str, user : &str, real : &str, chans : Vec < &str > ) -> IrcInfo {
    let mut cvec : Vec < String > = Vec::new();
    for chan in chans.iter( ) {
      cvec.push( String::from_str( *chan ) );
    }
    IrcInfo {
      nick_name : String::from_str( nick ),
      user_name : String::from_str( user ),
      real_name : String::from_str( real ),
      channels  : cvec,
      names     : collections::HashMap::new( ),
      prep_names: Vec::new( ),
    }
  }

  /// `update_info` is called whenever an event on the server affects us to see
  /// if anything in the client info has changed.
  ///
  /// # Arguments
  ///
  /// * `msg` - the raw message received from the server
  pub fn update_info( &mut self, msg : message::Message ) {
    match msg.code.as_slice( ) {
      // update nickname on NICK message
      "NICK" => {
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          self.nick_name = msg.param( 1 ).unwrap( ).to_string( );
        }
      },
      // add channels on JOIN message
      "JOIN" => {
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          if in_vec( &self.channels, msg.param( 1 ).unwrap( ).to_string( ) ).is_none( ) {
            self.channels.push( msg.param( 1 ).unwrap( ).to_string( ) );
          }
        } else {
          self.add_to_channel( msg.param( 1 ).unwrap( ).to_string( ), msg.nick( ).unwrap_or( String::from_str( "" ) ) );
        }
      },
      // remove channels on PART message
      "PART" => {
        if msg.nick( ).unwrap_or( String::from_str( "" ) ) == self.nick_name {
          match in_vec( &self.channels, msg.param( 1 ).unwrap( ).to_string( ) ) {
            Some( i ) => {
              self.drop_channel_names( msg.param( 1 ).unwrap( ).to_string( ) );
              self.channels.remove( i );
            },
            None      => (),
          }
        } else {
          self.remove_from_channel( msg.param( 1 ).unwrap( ).to_string( ), msg.nick( ).unwrap_or( String::from_str( "" ) ) );
        }
      },
      // remove channels on channel errors
      "403" | "405" | "437" | "471" | "473" | "474" | "475" | "476" => {
        match in_vec( &self.channels, msg.param( 1 ).unwrap( ).to_string( ) ) {
          Some( i ) => {
            self.drop_channel_names( msg.param( 1 ).unwrap( ).to_string( ) );
            self.channels.remove( i );
          },
          None      => (),
        }
      },
      _   => (),
    }
  }

  /// `prep_channel_names` parses a NAMES reply from the server and prepares to
  /// add it to a channel name list.
  ///
  /// # Arguments
  ///
  /// * `msg` - the raw message received from the server
  pub fn prep_channel_names( &mut self, msg : message::Message ) {
    let mut name_list = match msg.trailing( ) {
      Some( trail ) => trail.as_slice( ).trim_right( ).split_str( " " ),
      None          => return,
    };
    for name in name_list {
      self.prep_names.push( String::from_str( name ) );
    }
  }

  /// `set_channel_names` sets a channel's name list to the prepared names
  /// vector.
  ///
  /// # Arguments
  ///
  /// * `ch` - channel to set the name list on
  pub fn set_channel_names( &mut self, ch : String ) {
    let chan = strip_colon( &ch );
    // drop the name list if it already exists in our map
    if self.names.contains_key( &chan ) {
      self.drop_channel_names( chan.clone( ) );
    }

    // insert the name list into out map and get a pointer to it
    self.names.insert( chan.clone( ), Vec::new( ) );
    let chan_list = self.names.get_mut( &chan ).unwrap( );

    // populate the vector
    for name in self.prep_names.iter( ) {
      chan_list.push( name.clone( ) );
    }

    // clear our prep list
    self.prep_names.clear( );
  }

  /// `get_channel_names` returns the name list of a particular channel.
  ///
  /// # Arguments
  ///
  /// * `chan` - channel to get the name list for
  ///
  /// # Returns
  ///
  /// A String vector that contains the names of everyone on the given channel
  pub fn get_channel_names( &self, chan : String ) -> Option< &Vec < String > > {
    self.names.get( &chan )
  }

  /// `drop_channel_names` drops a channel's name list.
  ///
  /// # Arguments
  ///
  /// * `ch` - channel to drop the name list of
  fn drop_channel_names( &mut self, ch : String ) {
    // format our arguments
    let chan = strip_colon( &ch );
    
    // give a short debug message
    let debugline = format! ( "dropping {} from name lists", chan );
    debug::info( debugline.as_slice( ) );
    
    // get and clear the channel name list
    match self.names.get_mut( &chan ) {
      Some( list )  => list.clear( ),
      None          => {
        let eline = format! ( "name list '{}' does not exist", chan );
        debug::warn( "drop name list", eline.as_slice( ) );
        return;
      },
    }
    
    // remove the name list from our name map
    self.names.remove( &chan );
  }

  /// `add_to_channel` adds a nick to a channel's name list.
  ///
  /// # Arguments
  ///
  /// * `ch` - channel to add the name to
  /// * `ni` - nick to add to the channel name list
  fn add_to_channel( &mut self, ch : String, ni : String ) {
    // format our arguments
    let chan = strip_colon( &ch );
    let nick = strip_colon( &ni );
    
    // print a debug message
    let debugline = format! ( "adding {} to {}'s name list", nick, chan );
    debug::info( debugline.as_slice( ) );
    
    // get the channel name list and add the nick
    match self.names.get_mut( &chan ) {
      Some( list )  => list.push( nick ),
      None          => {
        let eline = format! ( "name list '{}' does not exist", chan );
        debug::warn( "add nick to name list", eline.as_slice( ) );
      },
    }
  }

  /// `remove_from_channel` removes a nick from a channel name list.
  ///
  /// # Arguments
  ///
  /// * `ch` - channel to remove the name from
  /// * `ni` - nick to remove from the channel name list
  fn remove_from_channel( &mut self, ch : String, ni : String ) {
    // format our arguments
    let chan = strip_colon( &ch );
    let nick = strip_colon( &ni );
    
    // print a debug message
    let debugline = format! ( "removing {} from {}'s name list", nick, chan );
    debug::info( debugline.as_slice( ) );
    
    // get the channel name list
    let chan_list = match self.names.get_mut( &chan ) {
      Some( list )  => list,
      None          => {
        let eline = format! ( "name list '{}' does not exist", chan );
        debug::warn( "remove nick from name list", eline.as_slice( ) );
        return;
      },
    };
    
    // loop through and remove the nick from the name list
    match in_vec( chan_list, nick ) {
      Some( i ) => { chan_list.remove( i ); },
      None      => (),
    }
  }
}

impl Drop for IrcInfo {
  fn drop ( &mut self ) {
    for (_,list) in self.names.iter_mut( ) {
      list.clear( );
    }
    self.names.clear( );
    self.channels.clear( );
  }
}

/// `in_vec` checks if an element is in a vector
///
/// # Arguments
///
/// * `v` - vector to look in
/// * `el` - element to look for
///
/// # Returns
///
/// The index of el if el is in v, otherwise None
fn in_vec < T > ( v : &Vec < T >, el : T ) -> Option < usize >
  where T: PartialEq {
  for i in range( 0, v.len( ) ) {
    if v[i] == el {
      return Some( i );
    }
  }
  return None;
}

/// `strip_colon` removes whitespace and colons from a string
///
/// # Arguments
///
/// * `s` - string to format
///
/// # Returns
///
/// A String with leading and trailing whitespaces and colons removed
///
/// # Notes
///
/// * This function is no longer necessary because message should strip most
/// whitespace
fn strip_colon ( s : &str::Str ) -> String {
  let ss = s.as_slice( ).trim( );
  if ss.starts_with( ":" ) {
    ss.slice_from( 1 ).to_string( )
  } else {
    ss.to_string( )
  }
}