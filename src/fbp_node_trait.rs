use std::thread;
use async_trait::async_trait;
use std::ops::{Deref};

use crate::fbp_iidmessage::*;
use crate::fbp_message_types::*;
use crate::fbp_nodecontext::*;
use crate::fbp_node_error::*;


/* --------------------------------------------------------------------------
    trait FBPNode

    Description:    The FBPNode trait defines the fundamental behaviors that 
                    a node must have.  

    Methods:
        node_data:

            Description:    This method will return the underlying 
                            FPCNodeContext for a node.

            Implementation:
                            This method MUST be implemented by all types that
                            adhere to this trait.

        node_data_mut:

            Description:    This method will return the underlying 
                            mutable FPCNodeContext for a node.

            Implementation:
                            This method MUST be implemented by all types that
                            adhere to this trait.
        process_config:

            Parameters: 
                msg:        The config message to be processed by this node.

            Description:    This method is the means of configuring a node.        
        
        process_message:

            Parameters: 
                msg:        The data message to be processed by this node.

            Description:    This method is the heart of a node.  It is where
                            messages are processed and the results of that 
                            processing are returned.  

        start:

            Description:    This method will start a thread that will start
                            by trying to get an IIDMessage to process.  If
                            there are no messages to process the thread will
                            block until a message is available.  This means
                            that the thread will only run if there is work
                            to be done.  Once a message is received it will
                            will be processed by the node's implementation 
                            of process_config if the message type is Config 
                            or process_message if the message type id Data.

            Implementation:
                            The default implementation of this method should
                            serve for most if not all nodes.  

        stop:

            Description:    This method will set the is_running atomic bool
                            to be false.  This will cause the loop started
                            in the start method to complete it's current 
                            processing and then fall out of the while loop.
   
            Implementation:
                            The default implementation of this method should
                            serve for most nodes.  Waits for thread to be 
                            joined before continuing.  This would be what most 
                            nodes would want to occur.  If not this method
                            can be re-implemented.
          
   -------------------------------------------------------------------------- */
   #[async_trait]
   pub trait FBPNode  : std::clone::Clone {
       fn node_data(&self) -> &FBPNodeContext;
   
       fn node_data_mut(&mut self) -> &mut FBPNodeContext;
   
       fn node_is_configured(&self) -> bool {
           self.node_data().node_is_configured()
       }
   
       async fn wait_for_node_to_be_configured(&self) {
           self.node_data().wait_for_node_to_be_configured().await;
       }
   
       fn do_process_message(&mut self, msg_to_process:IIDMessage) -> std::result::Result<(), NodeError> {

           let processed_msg = match msg_to_process.msg_type {
   
               MessageType::Data => {
                   if self.node_is_configured() {
                       if self.node_data().node_is_suspended() {
                           // The node is suspended.  Simply pass the 
                           // message along.
                           Ok(msg_to_process.clone())
                       } else {
                           // The node is operating let it process this
                           // message
                           self.process_message(msg_to_process.clone())
                       }
                   } else {
                      Err(NodeError::new("Node received a Data message BEFORE it was configured"))
                   }
               },
   
               MessageType::Process => {
                   if self.node_is_configured() {
                       self.process_process_message(msg_to_process.clone())
                   } else {
                       Err(NodeError::new("Node received a Process message BEFORE it was configured"))
                   }
               },
   
               MessageType::Config => {
                   self.process_config(msg_to_process.clone())
               },
   
               MessageType::Invalid => {
                   Ok(msg_to_process.clone())
               },
           };


           if processed_msg.is_ok()  {
                let msg_to_send = processed_msg.unwrap();

               if msg_to_send.msg_type != MessageType::Invalid {
                    let hashmap_ref = &self.node_data().output_vec.lock().unwrap();

                    for key in hashmap_ref.keys() {
                        for sender in  hashmap_ref.get(key).unwrap() {
                            let _ = sender.lock().unwrap().deref().input_queue.lock().unwrap().send(msg_to_send.clone());
                        }
                    }
               }
               return Ok(())
           }
   
           Err(processed_msg.err().unwrap())
       }
   
       // Config messages are used to "configure" a node BEFORE it is run.
       fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
           Ok(msg.clone())
       }
   
       // Process messages are used AFTER a node is running to effect the running node.
       fn process_process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
           if msg.payload.is_some() {
               let payload = msg.clone().payload.unwrap();
   
               let process_msg: ProcessMessage = ProcessMessage::make_self_from_string(payload.as_str());
               
               match process_msg.msg {
                   ProcessMessageType::Stop => {
                       if process_msg.message_node.is_some() {
                           if process_msg.message_node.unwrap() == self.node_data().uuid {
                               self.stop();
                           }
                       } else {
                           self.stop();
                       }
                   },
                   ProcessMessageType::Suspend => {
                       if process_msg.message_node.is_some() {
                           if process_msg.message_node.unwrap() == self.node_data().uuid {
                               self.node_data_mut().set_is_suspended(true);
                           }
                       } else {
                           // I THINK this needs to be specific
                           // self.node_data_mut().set_is_suspended(true);
                       }
                   },
                   ProcessMessageType::Restart => {
                       if process_msg.message_node.is_some() {
                           if process_msg.message_node.unwrap() == self.node_data().uuid {
                               if self.node_data().node_is_suspended() {
                                   self.node_data_mut().set_is_suspended(false);
                               }
                           } 
                       } else {
                           if self.node_data().node_is_suspended() {
                               self.node_data_mut().set_is_suspended(false);
                           }
                       }
                   },
               }
   
               if process_msg.propogate  == true {
                   return Ok(msg.clone())
               }
           }
           // Sending an invalid message so that it will NOT be propagated.
           Ok(IIDMessage::new(MessageType::Invalid, None))
       }
   
       // Process message is the 'normal' work processing for a node.
       fn process_message(&self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError>;
   
       /*
       fn debug_message_type(&self, a_msg: IIDMessage) {
           match a_msg.msg_type {
               MessageType::Data => println!("    The message is a Data message"),
               MessageType::Config => println!("    The message is a Config message"),
               MessageType::Process => println!("    The message is a Process message"),
               MessageType::Invalid => println!("    The message is an Invalid message"),
           }
       }
       */
   
       fn process_iter_msg(&self) {
   
           let receiver = self.node_data().rx.lock().unwrap();
           let iter = receiver.try_iter();
   
           for a_msg in iter {           
               // self.debug_message_type(a_msg.clone());
   
               let try_iter_message_result = self.process_message(a_msg.clone());
   
               if try_iter_message_result.is_err() {
                   println!("Error processing try_iter message {}", try_iter_message_result.err().unwrap());
               }
           }
       }
   
       fn start(self)  where Self: std::marker::Sized + Send + Sync + 'static {
   
           self.node_data().set_node_is_running(true);
   
           let mut the_node = self.clone();
   
           let _ = thread::spawn(move || {
               
               while the_node.node_data().node_is_running() {
   
                   // Always mark the node as quiescent BEFORE the blocking call to recv.
                   // If recv returns a message, then the node will be marked as processing.
   
                   let msg_to_process = the_node.node_data().rx.lock().unwrap().recv();
   
                   // The call to recv has returned. Mark the node as processing
                   if msg_to_process.is_ok() {
                       let a_msg = msg_to_process.unwrap();
   
                       let msg_processing_result = the_node.do_process_message(a_msg.clone());
   
                       if msg_processing_result.is_err() {
                           println!("Error processing message {}", msg_processing_result.err().unwrap())
                       }
                   }
               }
   
               // The while loop has completed.  This means that the node has 'stoppedc'
               // update the node context
               the_node.node_data().set_node_has_completed(true);
           });
       }
   
       fn stop(&self) {
           self.node_data().set_node_is_running(false);
       }
   }