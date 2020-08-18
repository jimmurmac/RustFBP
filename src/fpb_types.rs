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
use std::thread;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use std::fmt;
use std::error::Error;
use std::ops::{Deref};

// IIDMessage payload will be JSON strings in when this code is completed. 
//
// The serde::Deserialize, Serialize}; 
// use serde_json::Result;
//
// will allow for serializing Rust structs into a JSON string and back again. 



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
                    string in the field will be a JSON string obj in 'real'
                    life.
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

/* --------------------------------------------------------------------------
    Provide the ability to check equality for a ReceiverContext
   -------------------------------------------------------------------------- */

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
                    the output from a node. 

        is_running:
                    An Atomic boolean that specifies if a node is running and
                    is able to receive and process messages.
   -------------------------------------------------------------------------- */             

#[derive(Debug, Clone)]
pub struct FPBNodeContext { 
    name: &'static str,
    uuid: Uuid,
    tx: Arc<Mutex<Sender<IIDMessage>>>,
    rx: Arc<Mutex<Receiver<IIDMessage>>>,
    output_vec: Vec<Arc<Mutex<ReceiverContext>>>,
    is_running: Arc<AtomicBool>,
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
                            queue of messages that will be processed by 
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
        }
    }

    fn node_is_running(&self) -> bool {
        self.is_running.deref().load(Ordering::Relaxed)
    }

    fn set_node_is_running(&self, flag: bool) {
        self.is_running.store(flag, Ordering::Relaxed)
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

    Description:    The FPBNode trait defines the fundamental behaviors that 
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

pub trait FPBNode { 

    fn node_data(&self) -> &FPBNodeContext;

    fn node_data_mut(&mut self) -> &mut FPBNodeContext;  

    fn process_config(&self, msg:IIDMessage) -> std::result::Result<IIDMessage, NodeError>;

    fn process_message(&self, msg: IIDMessage) ->  std::result::Result<IIDMessage, NodeError>;

    fn start(self, a_node: Mutex<Arc<Box<dyn FPBNode + Send + Sync >>>) where Self: std::marker::Sized {
        thread::spawn( move || {
            let locked_node = a_node.lock().unwrap();
            if !locked_node.node_data().node_is_running() { 
                locked_node.node_data().set_node_is_running(true);
                while locked_node.node_data().node_is_running() { 
                    let msg_to_process = locked_node.node_data().rx.lock().unwrap().recv();
                    if msg_to_process.is_ok() {

                        let unwrapped_msg = msg_to_process.unwrap();

                        let processed_msg = match unwrapped_msg.msg_type {
                            MessageType::Config => {
                                locked_node.process_config(unwrapped_msg.clone())
                            },
                            MessageType::Data => {
                                locked_node.process_message(unwrapped_msg.clone())
                            },
                        };

                        if processed_msg.is_ok() {
                            let msg_to_send = processed_msg.unwrap();

                            for sender in &locked_node.node_data().output_vec {
                                let _ = sender.lock().unwrap().deref().input_queue.lock().unwrap().deref().send(msg_to_send.clone());
                            }
                        }   

                    } // if msg_to_process.is_ok()
                } // while locked_node.node_data().node_is_running()s
            } // if !locked_node.node_data().node_is_running()
        });
    }

    fn stop(&self) { 
       self.node_data().set_node_is_running(false);
    }

}


/* ==========================================================================
    Unit Test

    With Rust, Unit tests typically are in the same file as the 
    implementation. 
   ========================================================================== */

/* --------------------------------------------------------------------------
    mod test 

    Description:    Provide testing for the fpb_types file.

    Tests:
        test_iidmessage:

            Description:    Do testing for the struct IIDMessage. Creates a 
                            IIDMessage and ensures that fields are correct.

        test_fpbnode_context:

            Description:    Do testing for the struct FPBNodeContext. Creates
                            a FPBNodeContext and ensures that the fields are
                            correct.  Another FPBNodeContext is then created
                            and added as a receiver for the first node.  The
                            number of items in the output_vec are checked to
                            ensure that the receiver was added.  The receiver
                            is then removed and the output_vec is checked to
                            ensure that it is now empty.
   -------------------------------------------------------------------------- */
             
#[cfg(test)]
mod test {

    use super::MessageType;
    use super::IIDMessage;
    use super::FPBNodeContext;
    use super::NodeError;
    use super::FPBNode;
    use std::{thread, time};
    use std::fs::File;
    use std::io;
    use std::io::Read;
    use std::io::prelude::*;
    use std::sync::{Arc, Mutex};
    use std::fs::OpenOptions;

    /* ----------------------------------------------------------------------
        To allow for testing node interactions, this unit test creates three
        nodes:

        PassthroughNode:    This node is a "null" node.  It just passes it's
                            input to it's output

        AppendNode:         This node will append a string to the message's 
                            it receives and then sends them onto the next
                            node.

        LoggerNode:         This node will write out the payload of the 
                            data messages that are sent to it to a file.
       ---------------------------------------------------------------------- */

    #[derive(Debug, Clone)]
    struct PassthroughNode {

        // This needs to be a Arc<Mutex<FPBNodeContext>>
        // data: Arc<FPBNodeContext>,
        data: FPBNodeContext,
    }

    impl PassthroughNode {
        pub fn new(node_name: &'static str) -> Self {
            let pn = FPBNodeContext::new(node_name);

            PassthroughNode {
                //data: Arc::new(pn),
                data: pn,
            }
        }

        fn wrap_node(self) -> Mutex<Arc<Box<dyn FPBNode + Send + Sync>>> {
            Mutex::new(Arc::new(Box::new(self)))
        }
    }

    impl FPBNode for PassthroughNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn node_data_mut(&mut self) -> &mut FPBNodeContext {&mut self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<IIDMessage, NodeError> {

            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();

                if payload == "Stop".to_string() {
                    self.stop();
                }

                // What the AppendNode should do is have a message that would set the 
                // append_data from a Config message.  This would allow this node to be
                // dynamially configurable.  Given that this is only for a unit test,
                // I decided to be lazy.
                
            }

            // Sending the messsge on
            Ok(msg.clone())
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
            // Simply pass on the message that was sent to this node.

            Ok(msg.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct AppendNode {
       // data: Arc<FPBNodeContext>,
        data: FPBNodeContext,
        append_data: String,
    }


    impl AppendNode {
        pub fn new(node_name: &'static str) -> Self {
            let an = FPBNodeContext::new(node_name);
            let ad = String::new();

            AppendNode {
                //data: Arc::new(an),
                data: an,
                // append_data: Arc::new(ad),
                append_data: ad,
            }
        }

        pub fn set_append_data(&mut self, data: String) {
            self.append_data.clear();
            self.append_data.insert_str(0, data.as_str());
        }

        fn wrap_node(self) -> Mutex<Arc<Box<dyn FPBNode + Send + Sync>>> {
            Mutex::new(Arc::new(Box::new(self)))
        }
    }

    impl FPBNode for AppendNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn node_data_mut(&mut self) -> &mut FPBNodeContext {&mut self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<IIDMessage, NodeError> {

            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();
                if payload == "Stop".to_string() {
                    self.stop();
                }

                // What the AppendNode should do is have a message that would set the 
                // append_data from a Config message.  This would allow this node to be
                // dynamially configurable.  Given that this is only for a unit test,
                // I decided to be lazy.

                
            }

            Ok(msg.clone())
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {

            if msg.payload.is_some() {
                let mut payload = msg.clone().payload.unwrap();

                // Check to see if the append_data field has a value.  If it does then
                // append that value to the payload string.
                // NOTE:  In the 'real' world the payload string will be a JSON string.
                // this will allow for deserializing the JSON object into a Rust object.
                // the concept of 'append' would be quite different.  For now for this 
                // unit test, this will simply append the append data to the message 
                // payload and send it on.
                payload.push_str(self.append_data.as_str());
                let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
                return Ok(new_msg);
            }

            Ok(msg.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct LoggerNode{
        // data: Arc<FPBNodeContext>,
        data: FPBNodeContext,
        // log_file_name: Arc<String>,
        log_file_name: String,
    }
   
    impl LoggerNode {

        pub fn new(node_name: &'static str, logfile_name: &'static str) -> Self {

            let file = File::create(logfile_name).expect ("Unable to create file");
            drop(file);
            
            let lfn = String::from(logfile_name);
            let ln = FPBNodeContext::new(node_name);

            LoggerNode {
                //data: Arc::new(ln),
                data: ln,
                // log_file_name: Arc::new(lfn),
                log_file_name: lfn,
            }
        }  

        fn get_log_string(&mut self) -> io::Result<String> {

            let mut contents = String::new();

            let mut file = OpenOptions::new().read(true).open(self.log_file_name.as_str()).expect("Failed to open file {} for reading");

            file.read_to_string(&mut contents).expect("Failed to write contents to string");

            drop(file);
            
            Ok(contents)
        }

        fn log_string_to_file(&self, data: &String) -> io::Result<()> {

            let mut file = OpenOptions::new().append(true).open(self.log_file_name.as_str()).expect("Failed to open file for append");

            let _write_result = file.write(data.as_bytes());
            
            drop(file);

            Ok(())
        }

        fn wrap_node(self) -> Mutex<Arc<Box<dyn FPBNode + Send + Sync>>> {
            Mutex::new(Arc::new(Box::new(self)))
        }
    }

    impl FPBNode for LoggerNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn node_data_mut(&mut self) -> &mut FPBNodeContext {&mut self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<IIDMessage, NodeError> {
            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();
                if payload == "Stop".to_string() {
                    self.stop();
                }
            }

            Ok(msg.clone())
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
            if msg.payload.is_some() {

                let payload = msg.clone().payload.unwrap();

                if self.log_string_to_file(&payload).is_err() {
                    return Err(NodeError::new("Failed to write message to log file"));
                } 
            }

            Ok(msg.clone())
        }
    }

    #[test]
    fn test_iidmessage() {
        let msg = IIDMessage::new(MessageType::Data, Some("foo".to_string()));

        assert_eq!(msg.msg_type, MessageType::Data);
        assert_eq!(msg.payload.is_none(), false);
    }

    #[test]
    fn test_fpbnode_context() {
        let mut node = FPBNodeContext::new(&"TestNode");

        assert_eq!(node.name, "TestNode");
        assert_eq!(node.node_is_running(), false);

        let mut other_node = FPBNodeContext::new(&"OtherNode");

        node.add_receiver(&mut other_node);
        assert_eq!(node.output_vec.len(), 1);

        node.remove_receiver(&mut other_node);
        assert_eq!(node.output_vec.len(), 0);
    }

    #[test]
    fn run_node() {

        let mut a_passthrough_node =  PassthroughNode::new("PassthroughNode");

        let mut an_append_node = AppendNode::new("AppendNode");

        let mut a_log_node = LoggerNode::new("LoggerNode", "Log_file.txt");


        an_append_node.set_append_data(" World".to_string());

        // Wire of the nodes. NOTE: In the 'real' world, this configuration of 
        // a network would be accomplished by sending a Config message to a
        // 'Configurtor node that would read a JSON string that mapped the 
        // network and would create the nodes and wire them up.  In this simply
        // unit test, it will be done manually

        a_passthrough_node.data.add_receiver(&mut an_append_node.node_data_mut());
        an_append_node.data.add_receiver(&mut a_log_node.node_data_mut());

        // I might want to make a generic function that will do this AND start the node.
        let a_passthrough_node_fpbnode =  a_passthrough_node.clone().wrap_node();
        let an_append_node_fpbnode =  an_append_node.clone().wrap_node();
        let a_log_node_fpbnode =   a_log_node.clone().wrap_node();

        
         // Start the network NOTE: This shold most likely occur in the new function but
        // for now ...
        a_passthrough_node.clone().start(a_passthrough_node_fpbnode);
        an_append_node.clone().start(an_append_node_fpbnode);
        a_log_node.clone().start(a_log_node_fpbnode);
        

        // Wait for the last node to start.
        while !a_log_node.clone().node_data().node_is_running() {
            thread::sleep(time::Duration::new(1,0))
        }
 
        assert_eq!(a_passthrough_node.clone().node_data().node_is_running(), true);
        assert_eq!(an_append_node.clone().node_data().node_is_running(), true);
        assert_eq!(a_log_node.clone().node_data().node_is_running(), true);

        let my_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        a_passthrough_node.clone().node_data().post_msg(my_msg);

        thread::sleep(time::Duration::new(1,0));

        let my_stop_msg = IIDMessage::new(MessageType::Config, Some("Stop".to_string()));

        a_passthrough_node.clone().node_data().post_msg(my_stop_msg);

        thread::sleep(time::Duration::new(1,0));

        assert_eq!(a_passthrough_node.clone().node_data().node_is_running(), false);
        assert_eq!(an_append_node.clone().node_data().node_is_running(), false);
        assert_eq!(a_log_node.clone().node_data().node_is_running(), false);

        let log_string_result = a_log_node.clone().get_log_string();

        if log_string_result.is_ok() {
            let log_string = log_string_result.unwrap();

            assert_eq!(log_string, "Hello World");

        } 
    }

}




