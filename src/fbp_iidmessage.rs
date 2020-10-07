use crate::fbp_message_types::*;

/* --------------------------------------------------------------------------
    struct IIDMessage

    Define the standard data packet that will be sent to and from a node.  
   
    Fields:
        msg_type:   This field defines the type of message as defined by the
                    MessageType enum.

        payload:    This field will contain the actual message.  The type of
                    this field allows for an empty Message (None).  The 
                    string in the field will be a JSON string obj in 'real'
                    life.
   -------------------------------------------------------------------------- */

   #[derive(Debug, Clone, PartialEq, Eq)]
   pub struct IIDMessage {
       pub msg_type: MessageType,
       pub payload: Option<String>,
   }
   
   /* --------------------------------------------------------------------------
       struct IIDMessage behavior
   
       Methods:
           new:
               Parameters:
                   msg_type:   This will be the message type of the new message.
                   payload:    This will be the message for this message.  
   
               Description:    This will create a new message (Constructor)
       -------------------------------------------------------------------------- */
           
   #[allow(dead_code)] 
   impl IIDMessage {
       // Constructor for a IIDMessage
       pub fn new(msg_type: MessageType, payload: Option<String>) -> Self {
           IIDMessage {
               msg_type,
               payload,
           }
       }
   }