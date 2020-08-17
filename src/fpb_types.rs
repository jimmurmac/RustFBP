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
// use serde::{Deserialize, Serialize};
// use serde_json::Result;


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
            msg_type,
            payload,
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
   -------------------------------------------------------------------------- */             

#[derive(Debug, Clone)]
pub struct FPBNodeContext { 
    name: &'static str,
    uuid: Uuid,
    tx: Arc<Mutex<Sender<IIDMessage>>>,
    rx: Arc<Mutex<Receiver<IIDMessage>>>,
    output_vec: Vec<Arc<Mutex<ReceiverContext>>>,
    is_running: Arc<AtomicBool>,
    join_handle: Option<Arc<thread::JoinHandle<()>>>,
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
            join_handle: None,
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

pub trait FPBNode { 

    fn node_data(&self) -> &FPBNodeContext;

    // fn node_data_mut(&mut self) -> &mut FPBNodeContext;  

    fn process_config(&self, msg:IIDMessage) -> std::result::Result<(), NodeError>;

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

                        match unwrapped_msg.msg_type {
                            MessageType::Config => {
                                if locked_node.process_config(unwrapped_msg.clone()).is_err() {
                                    panic!("Failed a prosses_config: {}", unwrapped_msg.payload.unwrap_or("Unknown Message".to_string()));
                                }
                            },
                            MessageType::Data => {
                                let processed_msg = locked_node.process_message(unwrapped_msg.clone()); 
                                if processed_msg.is_ok() {
                                    let msg_to_send = processed_msg.unwrap().clone();
                                    for sender in &locked_node.node_data().output_vec {
                                        let _ = sender.lock().unwrap().deref().input_queue.lock().unwrap().deref().send(msg_to_send.clone());
                                    }
                                }
                            },
                        } // match unwrapped_msg.msg_type      
                    } // if msg_to_process.is_ok()
                } // while locked_node.node_data().node_is_running()s
            } // if !locked_node.node_data().node_is_running()
        });
    }


    fn stop(&self) { 
       self.node_data().set_node_is_running(false);
    }

}

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
    use std::path::Path;
    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::{Read, BufReader};
    use std::io::prelude::*;
    use std::sync::{Arc, Mutex};
    use std::fs::OpenOptions;


    #[derive(Debug, Clone)]
    struct PassthroughNode {
        data: Arc<FPBNodeContext>,
    }

    impl PassthroughNode {
        pub fn new(node_name: &'static str) -> Self {
            let pn = FPBNodeContext::new(node_name);

            PassthroughNode {
                data: Arc::new(pn),
            }
        }
    }

    impl FPBNode for PassthroughNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<(), NodeError> {
            // Nothing to do
            Ok(())
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
            // Simply pass on the message that was sent to this node.
            Ok(msg.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct AppendNode {
        data: Arc<FPBNodeContext>,
        append_data: Arc<Option<String>>,
    }


    impl AppendNode {
        pub fn new(node_name: &'static str) -> Self {
            let an = FPBNodeContext::new(node_name);

            AppendNode {
                data: Arc::new(an),
                append_data: Arc::new(None),
            }
        }

        pub fn set_append_data(&mut self, data: String) {
           let _ = self.append_data.replace(data);
        }
    }

    impl FPBNode for AppendNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<(), NodeError> {
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

            Ok(())
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
                if self.append_data.is_some() {
                    let append_string = self.append_data.as_ref().as_ref().unwrap();
                    payload.push_str(append_string.as_str());

                    let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
                    return Ok(new_msg);
                }                
            }

            Ok(msg.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct LoggerNode{
        data: Arc<FPBNodeContext>,
        //log_file:Arc<RwLock<File>>,
        log_file_name: Arc<String>,
    }
   
    impl LoggerNode {

        pub fn new(node_name: &'static str, logfile_name: &'static str) -> Self {

            if Path::new(logfile_name).exists() {
                fs::remove_file(logfile_name).expect("Faiiled to remove log file");
            }

            let _file = File::create(logfile_name).expect ("Unable to create file");
            
            let lfn = String::from(logfile_name);
            let ln = FPBNodeContext::new(node_name);

            LoggerNode {
                data: Arc::new(ln),
                //log_file: Arc::new(RwLock::new(f)), 
                log_file_name: Arc::new(lfn),

            }
        }  

        fn get_log_string(&mut self) -> io::Result<String> {
            let mut contents = String::new();

            let mut file = OpenOptions::new().read(true).open(self.log_file_name.as_str())?;
            
            let rr =   file.read_to_string(&mut contents)?;
            
            Ok(contents)
        }

        fn log_string_to_file(&self, data: &String) -> std::io::Result<()> {
            
            if !Path::new(self.log_file_name.as_str()).exists() {
                let _file = File::create(self.log_file_name.as_str())?;
            }

            let mut file = OpenOptions::new().append(true).open(self.log_file_name.as_str())?;
            file.write(data.as_bytes())?;
            Ok(())
        }

        pub fn wrap_self(self) -> Mutex<Arc<Box<dyn FPBNode + Send + Sync>>> {
            Mutex::new(Arc::new(Box::new(self.clone())))
        }
    }

    impl FPBNode for LoggerNode   {
        fn node_data(&self) -> &FPBNodeContext {&self.data}

        fn process_config(&self, msg:IIDMessage) -> Result<(), NodeError> {
            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();
                if payload == "Stop".to_string() {
                   self.stop();
                }
            }

            Ok(())
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();
                if self.log_string_to_file(&payload).is_err() {
                    return Err(NodeError::new("Failed to write message to log file"))
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
        assert_eq!(node.output_vec.len(), 0)
    }

    
   

    #[test]
    fn run_node() {

        let a_log_node = LoggerNode::new("LoggerNode", "Log_file.txt");

        let node =  a_log_node.clone().wrap_self();

        a_log_node.clone().start(node);

        while !a_log_node.clone().node_data().node_is_running() {
            thread::sleep(time::Duration::new(1,0))
        }

        assert_eq!(a_log_node.clone().node_data().node_is_running(), true);

        let my_msg = IIDMessage::new(MessageType::Data, Some("Test".to_string()));
        a_log_node.clone().node_data().post_msg(my_msg);

        a_log_node.clone().stop();

        let log_string_result = a_log_node.clone().get_log_string();
        if log_string_result.is_ok() {
            let log_string = log_string_result.unwrap();

            println!("In run_node and log_string = {}", log_string);

            assert_eq!(log_string, "Test"); 

        } else {
            println!("Failed to get the log string");
        }   
    }

}




