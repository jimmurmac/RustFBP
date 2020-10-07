use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fbp_iidmessage::*;


/* --------------------------------------------------------------------------
    enum MessageType

    Provide a means of differentiating different types of messages sent to 
    nodes.
   -------------------------------------------------------------------------- */

   #[derive(Debug, Copy, Clone, PartialEq, Eq)]
   pub enum MessageType {
       Data,       // This is a 'normal' data packet to be processed by a node
       Config,     // This is a configuration packet used to configure a node
       Process,    // This is a process message to the node, stop is an example
       Invalid,    // This is not a valid message.  It should not be propagated 
                   // to the next node.
   }
   
   /* --------------------------------------------------------------------------
       trait MessageSerializer
   
       Description:    The MessageSerializer trait allows for serializing Rust
                       structs that are used for IIDMessages.  It also allows 
                       for taking a JSON string and recreating the Rust struct.
                       This trait should be implemented by all Rust structs that
                       are used as the basis for creating IIDMessages.  The
                       good news is that the trait already implements all of the
                       necessary behavior.  A Rust struct that is used for
                       IIDMessages need only add the following line to adhere to
                       this trait.
   
                       impl MessageSerializer for MyMessageStruct{}
   
       Methods:
           make_self_from_string:
   
               Parameters:
                   json_string:
                               A Rust str that is a JSON string that was
                               created by the make_message method.
   
               Description:    Given a JSON string that was created by
                               serializing a Rust Message Struct, this
                               method will create a new Rust struct that
                               matches the original Rust Message Struct.
   
               Implementation:
                               This method is fully implemented by this
                               trait. A Rust Message struct need only
                               add the single impl MessageSerializer for
                               the type.
   
   
           make_message:
   
                Parameters:
                   msg_type:   The type of Message that the IIDMessage
                               should have.  Please see the MessageType
                               enumeration of the types of IIDMessage
                               that can be sent.
   
               Description:    This method will take a Rust Message Struct
                               and turn it into an IIDMessage that can be
                               sent to a node for processing.
   
                Implementation:
                               This method is fully implemented by this
                               trait. A Rust Message struct need only
                               add the single impl MessageSerializer for
                               the type.
      -------------------------------------------------------------------------- */
   pub trait MessageSerializer {
   
       fn  make_self_from_string<'a>(json_string: &'a str) -> Self
           where Self: std::marker::Sized + serde::Deserialize<'a> {
           serde_json::from_str(json_string).unwrap()
       }
   
       fn make_message(&self, msg_type: MessageType) -> IIDMessage
           where Self: std::marker::Sized +  serde::Serialize {
           let struct_string = serde_json::to_string(&self).unwrap();
           IIDMessage::new( msg_type, Some(struct_string))
       }
   }
   
   /* --------------------------------------------------------------------------
       enum ConfigMessageType
   
       Provide a means of differentiating different types of config messages sent
       to nodes.
      -------------------------------------------------------------------------- */
   
   #[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
   pub enum ConfigMessageType {
       Connect,    // Connect the output of a node to the input of another
                   // node. Note: This will require  arguments to complete
       Disconnect, // Disconnect the output of a node from the input of
                   // another node.  This will require  arguments to complete
       Map,        // Map the node network.  This will require  arguments
                   // to complete
       Move,       // Move a node in this process to either another process
                   // or another machine.  This will require  arguments
                   // to complete
       Delete,     // Delete a specific node in the network.  This will 
                   // require  arguments to complete
       Field,      // Set a value field in a node.
   }
   
   /* --------------------------------------------------------------------------
       struct ConfigMessage
   
       Define the standard data struct that will be serialized into a JSON string
       and sent in an IIDMessage struct.  
   
       Fields:
           msg:        This is a ConfigMessageType.  It defines what type of 
                       configuration message this is.
   
           data:       This is an optional data string (JSON) of the data for
                       to used to configure a node
      -------------------------------------------------------------------------- */
   #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
   pub struct ConfigMessage {
       pub msg: ConfigMessageType,
       pub data: Option<String>,
   }
   
   impl MessageSerializer for ConfigMessage{}
   
   /* --------------------------------------------------------------------------
       enum ProcessMessageType
   
       Provide a means of differentiating different types of process messages sent
       to nodes.
      -------------------------------------------------------------------------- */
   #[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
   pub enum ProcessMessageType {
       Stop,       // Stop a running node
       Suspend,    // Tell node to stop processing data messages.  The node
                   // just pass on data messages on changed after receiving this
                   // message
       Restart,    // Restart a node that was previously suspended
   }
   
   /* --------------------------------------------------------------------------
       struct ProcessMessage
   
       Define the standard data struct that will be serialized into a JSON string
       and sent in an IIDMessage struct.  
   
       Fields:
           msg:        This is a ProcessMessageType.  It defines what type 
                       processing should be done.
   
           propogate:  This bool defines if the message should be sent on to all
                       of the output nodes for the receiver.
      -------------------------------------------------------------------------- */
   #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
   pub struct ProcessMessage {
       pub msg: ProcessMessageType,
       pub propogate: bool,
       pub message_node: Option<Uuid>,
   }
   
   #[allow(dead_code)]
   impl ProcessMessage {
       pub fn new(msg: ProcessMessageType, propogate: bool) -> Self {
           ProcessMessage {
               msg,
               propogate,
               message_node: None,
           }
       }
   
       pub fn set_specific_node_receiver(&mut self, uuid: Uuid) {
           self.message_node = Some(uuid);
       }
   }
   
   impl MessageSerializer for ProcessMessage{}

   