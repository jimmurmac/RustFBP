/* ==========================================================================
    File:           fpb_types.rs

    Description:    This file defines the basic types and behavior needed to 
                    make a Flow Based Programming model 
                    (https://jpaulm.github.io/fbp/index.html) in a process.  
                   
    History:        Jim Murphy 08/07/2020 Initial Code.

    Copyright ©  2020 Pesa Switching Systems Inc. All rights reserved.
   ========================================================================== */

// System Libraries (Crates) used by this file
use std::sync::mpsc::{Sender, Receiver, channel};
use std::{thread, time};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use std::fmt;
use std::error::Error;
use std::ops::Deref;

/* --------------------------------------------------------------------------
    enum MessageType

    Provide a means of differentiating different types of messages sent to 
    nodes.
   -------------------------------------------------------------------------- */

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Data,       // This is a 'normal' data packet to be processed by a node
    Config,     // This is a configuration packet used to configure a node
}

/* --------------------------------------------------------------------------
    struct IIDMessage

    Define the standard data packet that will be sent to and from a node.  
   
    Fields:
        msg_type:   This field defines the type of message as defined by the
                    MessageType enum.

        payload:    This field will contain the actual message.  The type of
                    this field allows for an empty Message (None).  The 
                    string in the field will most likely be a JSON string obj
   -------------------------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IIDMessage {
    msg_type: MessageType,
    payload: Option<String>,
}

/* --------------------------------------------------------------------------
    struct IIDMessage behavior

    Methods:
        new:
            Parameters:
                msg_type:   This will be the message type of the new message.
                payload:    This will be the message for this message.  

            Description:    This will create a new message (Constructor)
        
        msg_type:
            Description:    Accessor method to get the message type of this 
                            message.

        payload:
            Description:    Accessor method to get the payload as a reference
                            for this message.
    -------------------------------------------------------------------------- */
        
#[allow(dead_code)] 
impl IIDMessage {
    // Constructor for a IIDMessage
    pub fn new(msg_type: MessageType, payload: Option<String>) -> Self {
        IIDMessage {
            msg_type: msg_type,
            payload: payload,
        }
    }

    // Return the message type for a IIDMessage
    pub fn msg_type(&self) -> MessageType {
        self.msg_type
    }

    // Return a reference to the payload of a IIDMessage
    pub fn payload(&self) -> &Option<String> {
        &self.payload
    }
}

/* --------------------------------------------------------------------------
    struct ReceiverContext

    Define the record that contains the "socket" to send the output of a node
    to another node.

    Fields:
        node_uuid:  The unique identifier of the node that will be receiving
                    the out of a node.

        input_queue:   
                    This field if the asynchronous input channel for a node.
                    This will allow a node to queue a message into another
                    node for processing.
   -------------------------------------------------------------------------- */

#[derive(Debug, Clone)]
struct ReceiverContext {
    node_uuid: Uuid,
    input_queue: Arc<Mutex<Sender<IIDMessage>>>,
}

/* --------------------------------------------------------------------------
    struct ReceiverContext behavior

    Methods:
         new:
            Parameters: 
                node:       A mutable reference to the FPBNodeContex that 
                            will receive the output from a node.
            
            Description:    This will create a new ReceiverContext 
                            (Constructor)
   -------------------------------------------------------------------------- */

impl ReceiverContext {
    pub fn new(node: &mut FPBNodeContext) -> Self {
        ReceiverContext {
            node_uuid: node.uuid.clone(),
            input_queue: node.tx.clone(),
        }
    }
}

impl PartialEq for ReceiverContext {
    fn eq(&self, other: &Self) -> bool {
        self.node_uuid == other.node_uuid
    }
}

/* --------------------------------------------------------------------------
    struct FPBNodeStruct

    Defines the basic struture of a FPB node.

     Fields:
        name:       This is the name associated with the node.  NOTE:  This 
                    field can be blank (None)

        uuid:       This is a unique ID for this node.  It is how a specific
                    instantiation of a node is identified.

        tx:         The tx field is the sender end of an 
                    asynchronous channel.  
                    https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html

        rx:         The rx field is the receiver end of an asynchronous
                    channel.

        output_vec:
                    A vector of sender channels for the nodes that wish to 
                    get the output of the processing for this node.  Given
                    that this is a vector, any number of nodes can receive 
                    the output of a node.  This will allow for both real 
                    processing of a series of messages as well as logging.

        is_running:
                    An Atomic boolean that specifies if a node is running and
                    is able to receive and process messages.

        is_joined:  An Atomic boolean that specifies if a node has been 
                    stopped and that the join has completed.

   -------------------------------------------------------------------------- */             

#[derive(Debug, Clone)]
pub struct FPBNodeContext { 
    name: &'static str,
    uuid: Uuid,
    tx: Arc<Mutex<Sender<IIDMessage>>>,
    rx: Arc<Mutex<Receiver<IIDMessage>>>,
    output_vec: Vec<Arc<Mutex<ReceiverContext>>>,
    is_running: Arc<AtomicBool>,
    is_joined: Arc<AtomicBool>,
}

/* --------------------------------------------------------------------------
    struct FPBNodeContext behavior

    Methods:
        new:
            Parameters: 
                name:       The name to be associated with this new node. 
                            NOTE: This make be empty (None)
            
            Description:    This will create a new FPBNodeContext 
                            (Constructor)

        node_is_running:
            Description:    Returns true if the node processing has been 
                            started, false otherwise.

            Returns:        Boolean, true if the node is processing or 
                            false if the node is NOT processing.

        set_node_is_running:
            Parameters:
                flag:       A boolean that will be set for the AtomicBool
                            member to specify if the node processing has
                            started (true) or if the node is NOT processing
                            (false)    
                                            

        node_is_joined:
            Description:    Returns true if the node processing thread 
                            has been joined, false otherwise.

            Returns:        Boolean, true if the node has been joined 
                            after it was stopped, false otherwise.


        set_node_is_joined:
            Parameters:
                flag:       A boolean that will be set for the AtomicBool
                            member to specify if the node thread has been
                            joined.

        add_receiver:
            Parameters:
                receiver:   A mutable reference to a FPBNodeContext that 
                            wants to receive the output from the processing
                            of this node. This receiver will be added to a
                            vector of receivers for this node.

        remove_receiver:
            Parameters:
                receiver    A mutable reference to a FPBNodeContext that 
                            no longer wishes to receive the output of this 
                            node.

        post_msg:
            Parameters:
                msg:        An input IIDMessage that will be added to the 
                            queue of messages that need to be processed by 
                            this node.

   -------------------------------------------------------------------------- */

#[allow(dead_code)]
impl FPBNodeContext {

    fn new(name: &'static str) -> Self {

        let (sender, receiver) = channel::<IIDMessage>();

        FPBNodeContext {
            name,
            uuid: Uuid::new_v4(),
            tx: Arc::new( Mutex::new(sender)),
            rx: Arc::new(Mutex::new(receiver)),
            output_vec: Vec::new(),
            is_running: Arc::new(AtomicBool::new(false)),
            is_joined: Arc::new(AtomicBool::new(false)),

        }
    }

    fn node_is_running(&self) -> bool {
        self.is_running.deref().load(Ordering::Relaxed)
    }

    fn set_node_is_running(&self, flag: bool) {
        self.is_running.store(flag, Ordering::Relaxed)
    }

    fn node_is_joined(&self) -> bool {
        self.is_joined.deref().load(Ordering::Relaxed)
    }

    fn set_node_is_joined(&self, flag: bool) {
        self.is_joined.store(flag, Ordering::Relaxed)
    }

    pub fn add_receiver(&mut self, receiver: &mut FPBNodeContext ) {
        let rr = ReceiverContext::new(receiver);
        self.output_vec.push(Arc::new(Mutex::new(rr)));
    }

    pub fn remove_receiver(&mut self, receiver: &mut FPBNodeContext) {
        let rr = ReceiverContext::new(receiver);
        let index = self.output_vec.iter().position(|r| r.lock().unwrap().deref() == &rr).unwrap();
        self.output_vec.remove(index);
    }

    pub fn post_msg(&self, msg: IIDMessage) {
        if self.node_is_running() {
            let _ = self.tx.lock().unwrap().deref().send(msg);
        }
    }
}

/* --------------------------------------------------------------------------
    struct NodeError

    Defines an error type for nodes.  This is still a Work in Progress.  
    What needs to be done is to provide an enumeration of various node errors.

    Fields:
        details:    The details field will contain a human readable version 
                    of the error.
   -------------------------------------------------------------------------- */

#[derive(Debug)]
pub struct NodeError {
    details: String
}

/* --------------------------------------------------------------------------
    struct NodeError behavior

    Methods:
        new:
            Parameters: 
                msg:        A human readable error message.

            Description:    This will create a new NodeError 
                            (Constructor)
   -------------------------------------------------------------------------- */

#[allow(dead_code)]
impl NodeError {
    fn new(msg: &str) -> NodeError {
        NodeError {details: msg.to_string()}
    }
}

/* --------------------------------------------------------------------------
    Implement the fmt::Display trait for the NodeError struct
   -------------------------------------------------------------------------- */

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

/* --------------------------------------------------------------------------
    Implement the Error trait for the NodeError struct
   -------------------------------------------------------------------------- */

impl Error for NodeError {
    fn description(&self) -> &str {
        &self.details
    }
}

/* --------------------------------------------------------------------------
    trait FPBNode

    Description:    The FPBNode trait defines the fundamental behavior that 
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
        
        process_message:

            Parameters: 
                msg:        The message to be processed by this node.

            Description:    This method is the heart of a node.  It is where
                            messages are processed and the results of that 
                            processing are returned.  

            Implementation:
                            The default implementation of this method simply
                            clones the incoming message and returns it.  
                            Almost all types that wish to adhere to this 
                            trait, will want to re-implement this method so
                            that it preforms the work of the node.

        start:

            Description:    This method will start the processing for a node.
                            It does this my stipulating that the node is 
                            processing and then spawns a thread to read from
                            the input queue of messages and then process 
                            those messages until the node is stopped.  NOTE:
                            If there are no messages in the input queue the
                            call to recv will block until a message is 
                            available.  This means that unless a node has
                            work to do it is quiescent.  

            Implementation:
                            The default implementation of this method should
                            serve for most if not all nodes.  

        stop:

            Description:    This method will set the is_running atomic bool
                            to be false.  This will cause the loop started
                            in the start method to complete it's current 
                            processing and then fall out of the while loop
                            and have the thread joined so as not to leave 
                            zombies.  

            
            Implementation:
                            The default implementation of this method should
                            serve for most nodes.  Waits for thread to be 
                            joined before continuing.  This would be what most 
                            nodes would want to occur.  If not this method
                            can be re-implemented.
          
   -------------------------------------------------------------------------- */

pub trait FPBNode { // :  FPBNodeData   {

    fn node_data(&self) -> FPBNodeContext;

    fn node_data_mut(&mut self) -> &mut FPBNodeContext;

    fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
        Ok(msg.clone())
    }

    fn start(self)  where Self: std::marker::Sized, Self: std::marker::Send, Self: 'static {

        if !self.node_data().node_is_running() {
            let node = self.node_data();
            let join_node = node.clone();

            let child = thread::spawn(move || {
                node.set_node_is_running(true);

                while node.node_is_running() {
                    let msg_to_process = node.rx.lock().unwrap().recv();
                    if msg_to_process.is_ok() {
                        let msg = msg_to_process.unwrap().clone();
                        let processed_msg = self.process_message(msg);
                        if processed_msg.is_ok() {
                            let msg_to_send = processed_msg.unwrap().clone();
                            for sender in node.output_vec.clone() {
                                let _ = sender.clone().lock().unwrap().deref().input_queue.lock().unwrap().deref().send(msg_to_send.clone());
                            }
                        }
                    }
                }
            });

            child.join().expect("Could not join thread for Node");
            join_node.set_node_is_joined(true);
        }
    }

    fn stop(&mut self) {
       self.node_data().set_node_is_running(false);
       while !self.node_data().node_is_joined() {
           thread::sleep(time::Duration::new(1,0)) // This waits for 1 second.  TODO: maybe wait for shorter time.
       }
    }

}
/*

#[cfg(test)]
mod test {

    use super::IIDMessage;
    use super::MessageType;
    use super::FPBNodeStruct;
    use super::NodeState;

    #[test]
    fn type_test() {
        let msg = IIDMessage::new(MessageType::Data, Some("foo".to_string()));

        assert_eq!(msg.msg_type, MessageType::Data);
        assert_eq!(msg.payload.is_none(), false);

        let node = FPBNodeStruct::new(Some("Bob"));

        assert_eq!(node.name, Some("Bob"));
        assert_eq!(node.state, NodeState::Quiescent);
        assert_eq!(node.node_thread.is_none(), true);
    }
}

*/


