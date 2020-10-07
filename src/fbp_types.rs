/* ==========================================================================
    File:           fbp_types.rs

    Description:    This file defines the basic types and behavior needed to 
                    make a Flow Based Programming model 
                    (https://jpaulm.github.io/fbp/index.html) in a process.  
                   
    History:        Jim Murphy 08/07/2020   Initial Code.
                    Jim Murphjy 09/13/2020  Got all of the unit tests passing.

    Copyright ©  2020 Jim Murphy All rights reserved.
   ========================================================================== */

// System Libraries (Crates) used by this file
// #![allow(unused_imports)]
use std::sync::mpsc::{Sender, Receiver, channel};
use std::{thread};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use std::fmt;
use std::error::Error;
use std::ops::{Deref};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{self, Visitor, SeqAccess, MapAccess};
use serde::ser::{SerializeStruct};

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
    msg: ProcessMessageType,
    propogate: bool,
}

#[allow(dead_code)]
impl ProcessMessage {
    pub fn new(msg: ProcessMessageType, propogate: bool) -> Self {
        ProcessMessage {
            msg,
            propogate,
        }
    }
}

impl MessageSerializer for ProcessMessage{}


/* --------------------------------------------------------------------------
    struct ConfigNode


    Defines the arguments needed for a ConfigNode.  A Config node defines
    the configuration of node.  This is used by the configurator to define
    nodes.

    Fields:
        node_name:          This is the name of the node type to be created.

        configurations:     This is a vector of the parameters that this 
                            node needs in order to be 'configured'

        connections:alloc   This is a network of nodes that this node will
                            need to receive the output of this node.
   -------------------------------------------------------------------------- */

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfigNode {
    pub node_name: String,
    pub configurations: Vec<String>,
    pub connections: ConfigNodeNetwork,
}

#[allow(dead_code)]
impl ConfigNode {

    pub fn new(node_name: String) -> Self {
        ConfigNode {
            node_name,
            configurations: Vec::new(),
            connections: ConfigNodeNetwork {
                node_network: Vec::new(),
            },
        }
    }

    pub fn add_configuration(&mut self, config_str:String) {
        let configs = &mut self.configurations;
        configs.push(config_str);
        // self.configurations.push(config_str);
    }

    pub fn add_connection(&mut self, node_name: String) ->  Option<&mut ConfigNode> {
        let config_node = self.connections.add_node(node_name);
        config_node
    }
}

impl MessageSerializer for ConfigNode{}

/* --------------------------------------------------------------------------
    struct ConfigNodeNetwork


    Defines the nodes that need to receive the output from a node.

    Fields:
        node_network:       This is a vector of nodes to receive the output
                            of a node.
   -------------------------------------------------------------------------- */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfigNodeNetwork {
    pub node_network: Vec<ConfigNode>,
}

#[allow(dead_code)]
impl ConfigNodeNetwork {

    pub fn new() -> Self {
        ConfigNodeNetwork {
            node_network: Vec::new(),
        }
    }

    pub fn find_node(&mut self, node_name: String) -> Option<&mut ConfigNode> {

        let mut result: Option<&mut ConfigNode> = None;

        for a_config_node in &mut self.node_network {
            if a_config_node.node_name == node_name {
                result = Some(a_config_node);
                break;
            }
        }

        result
    }

    pub fn add_node(&mut self, node_name: String) -> Option<&mut ConfigNode> {
        // First create the node.
        let nn = node_name.clone();
        let node_config = ConfigNode {
            node_name,
            configurations: Vec::new(),
            connections: ConfigNodeNetwork {
                node_network: Vec::new(),
            },
        };

        self.node_network.push(node_config);
        self.find_node(nn)
    }
}

impl MessageSerializer for ConfigNodeNetwork{}


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
    node_uuid: Uuid,
    input_queue: Arc<Mutex<Sender<IIDMessage>>>,
    node_type: String,
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

/* --------------------------------------------------------------------------
    struct FBPNodeStruct

    Defines the basic struture of a FBP node.

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
pub struct FBPNodeContext { 
    pub name: String,
    pub uuid: Uuid,
    tx: Arc<Mutex<Sender<IIDMessage>>>,
    rx: Arc<Mutex<Receiver<IIDMessage>>>,
    pub output_vec: Arc<Mutex<Vec<Arc<Mutex<ReceiverContext>>>>>,
    pub is_running: Arc<AtomicBool>,
    pub node_completion: Arc<AtomicBool>,
}

/* --------------------------------------------------------------------------
    struct FBPNodeContext behavior

    Methods:
        new:
            Parameters: 
                name:       The name to be associated with this new node. 
                            NOTE: This make be empty (None)
            
            Description:    This will create a new FBPNodeContext 
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
                receiver:   A mutable reference to a FBPNodeContext that 
                            wants to receive the output from the processing
                            of this node. This receiver will be added to a
                            vector of receivers for this node.

        remove_receiver:
            Parameters:
                receiver    A mutable reference to a FBPNodeContext that 
                            no longer wishes to receive the output of this 
                            node.

        post_msg:
            Parameters:
                msg:        An input IIDMessage that will be added to the 
                            queue of messages that will be processed by 
                            this node.
   -------------------------------------------------------------------------- */

#[allow(dead_code)]
impl FBPNodeContext {

    pub fn new(name: &str) -> Self {

        let (sender, receiver) = channel::<IIDMessage>();

        FBPNodeContext {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            tx: Arc::new( Mutex::new(sender)),
            rx: Arc::new(Mutex::new(receiver)),
            output_vec: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            node_completion:  Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn node_is_running(&self) -> bool {
        self.is_running.deref().load(Ordering::Relaxed)
    }

    fn set_node_is_running(&self, flag: bool) {
        self.is_running.store(flag, Ordering::Relaxed)
    }

    pub fn node_has_completed(&self) -> bool {
        self.node_completion.deref().load(Ordering::Relaxed)
    }

    fn set_node_has_completed(&self, flag: bool) {
        self.node_completion.store(flag,Ordering::Relaxed)
    }

    pub fn add_receiver(&mut self, receiver: &mut FBPNodeContext ) {
        let rr = ReceiverContext::new(receiver);
        self.output_vec.lock().unwrap().push(Arc::new(Mutex::new(rr)));
    }

    pub fn remove_receiver(&mut self, receiver: &mut FBPNodeContext) {
        let rr = ReceiverContext::new(receiver);
        let index = self.output_vec.lock().unwrap().iter().position(|r| r.lock().unwrap().deref() == &rr).unwrap();
        self.output_vec.lock().unwrap().remove(index);
    }

    pub fn post_msg(&self, msg: IIDMessage) {
        if self.node_is_running() {
            let _ = self.tx.lock().unwrap().deref().send(msg);
        }
    }
}

/* --------------------------------------------------------------------------
    Serialize for a FBPNodeContext struct

    Most of the fields in a FBPNodeContext struct are only part of an
    existing network of nodes.  In order to allow for making a JSON
    representation of a network of nodes, the FBPNodeContext is part of
    all nodes.  To be able to serialize an existing network, a node will
    need to be able to serialize the FBPNodeContext struct.  This
    implementation allows for serializing a FBPNodeContext.
   -------------------------------------------------------------------------- */

impl Serialize for FBPNodeContext {

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer,
    {
        //serializer.serialize_str(self.name)
        let mut state = serializer.serialize_struct("FBPNodeContext", 1)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

/* --------------------------------------------------------------------------
    Deserialize for a FBPNodeContext struct

    Once a FBPNodeContext struct has been serialized, there needs to be a way
    to take the JSON string and turn it back into a FBPNodeContext. That is
    what this implementation does.
   -------------------------------------------------------------------------- */
impl<'de> Deserialize<'de> for FBPNodeContext {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {

        enum Field { Name/*, Tx, Rx, Output_vec, Is_running */};

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("name")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct FBPNodeContextVisitor;

        impl<'de> Visitor<'de> for FBPNodeContextVisitor {
            type Value = FBPNodeContext;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct FBPNodeContext")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<FBPNodeContext, V::Error>
                where V: SeqAccess<'de>
            {
                let name: String = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(FBPNodeContext::new(name.as_str()))
            }

            fn visit_map<V>(self, mut map: V) -> Result<FBPNodeContext, V::Error>
                where V: MapAccess<'de>
            {
                let mut name: Option<String> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                Ok(FBPNodeContext::new(name.as_str()))
            }
        }

        const FIELDS: &'static [&'static str] = &["name"];
        deserializer.deserialize_struct("FBPNodeContext", FIELDS, FBPNodeContextVisitor)
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
    pub fn new(msg: &str) -> NodeError {
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
pub trait FBPNode  : std::clone::Clone {
    fn node_data(&self) -> &FBPNodeContext;

    fn node_data_mut(&mut self) -> &mut FBPNodeContext;

    fn node_is_configured(&self) -> bool;

    fn do_process_message(&mut self, msg_to_process:IIDMessage) -> std::result::Result<(), NodeError> {

        let processed_msg = match msg_to_process.msg_type {

            MessageType::Data => {
                if self.node_is_configured() {
                    self.process_message(msg_to_process)
                } else {
                   Err(NodeError::new("Node received a Data message BEFORE it was configured"))
                }
            },

            MessageType::Process => {
                if self.node_is_configured() {
                    self.process_process_message(msg_to_process)
                } else {
                    Err(NodeError::new("Node received a Process message BEFORE it was configured"))
                }
            },

            MessageType::Config => {
                self.process_config(msg_to_process)
            },

            MessageType::Invalid => Ok(msg_to_process),
        };

        if processed_msg.is_ok()  {
            let msg_to_send = processed_msg.unwrap();
            if msg_to_send.msg_type != MessageType::Invalid {
                for sender in self.node_data().output_vec.lock().unwrap().deref() {
                    let _ = sender.lock().unwrap().deref().input_queue.lock().unwrap().deref().send(msg_to_send.clone());
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
    fn process_process_message(&self, msg: IIDMessage) -> std::result::Result<IIDMessage, NodeError> {
        if msg.payload.is_some() {
            let payload = msg.clone().payload.unwrap();
            let process_msg: ProcessMessage = ProcessMessage::make_self_from_string(payload.as_str());

            if process_msg.msg == ProcessMessageType::Stop {
                self.stop();
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

        let mut the_node= self.clone();

        let _ = thread::spawn(move || {

            while the_node.node_data().node_is_running() {
                let msg_to_process = the_node.node_data().rx.lock().unwrap().recv();

                if msg_to_process.is_ok() {
                    let a_msg = msg_to_process.unwrap();

                    // the_node.debug_message_type(a_msg.clone());

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


