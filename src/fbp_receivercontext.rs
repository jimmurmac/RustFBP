use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::sync::mpsc::{Sender};

use crate::fbp_iidmessage::*;
use crate::fbp_nodecontext::*;




/* --------------------------------------------------------------------------
    struct ReceiverContext

    Define the record that contains the "socket" to send the output of a node
    to another node.

    Fields:
        node_uuid:  The unique identifier of the node that will be receiving
                    the output of another node.

        input_queue:   
                    This field is the asynchronous input channel for a node.
                    This will allow a node to queue a message to another
                    node for processing.
   -------------------------------------------------------------------------- */

   #[derive(Debug, Clone)]
   pub struct ReceiverContext {
       pub node_uuid: Uuid,
       pub input_queue: Arc<Mutex<Sender<IIDMessage>>>,
       pub node_type: String,
   }
   
   /* --------------------------------------------------------------------------
       struct ReceiverContext behavior
   
       Methods:
            new:
               Parameters: 
                   node:       A mutable reference to the FBPNodeContex that 
                               will receive the output from a node.
               
               Description:    This will create a new ReceiverContext 
                               (Constructor)
      -------------------------------------------------------------------------- */
   
   impl ReceiverContext {
       pub fn new(node: &mut FBPNodeContext) -> Self {
           ReceiverContext {
               node_uuid: node.uuid.clone(),
               input_queue: node.tx.clone(),
               node_type: node.name.clone(),
           }
       }
   }
   
   /* --------------------------------------------------------------------------
       Provide the ability to check equality for a ReceiverContext
      -------------------------------------------------------------------------- */
   
   impl PartialEq for ReceiverContext {
       fn eq(&self, other: &Self) -> bool {
           self.node_uuid == other.node_uuid
       }
   }