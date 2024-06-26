/* =================================================================================
 File:           fbp_node_trait.rs

 Description:    This file defines a trait that all FBP nodes must adhere to in
                 order to be an FBP node

 History:        RustDev 03/31/2021   Code ported from original rustfbp crate

 Copyright ©  2021 Pesa Switching Systems Inc. All rights reserved.
================================================================================== */
#![allow(static_mut_refs)]
//! # FBP Node Trait
//!
//! This trait provides the functionality of an FBP node.  All structs that are to be
//! FBP nodes **must** implement this trait.  The good news is that most of the involved
//! behaviors have already been implemented by this trait.
//!
//! The following is an example of a simple Flow Based Programming system using three FBP nodes
//!
//!
//! # Example
//!
//! ```
//!use serde::{Deserialize, Serialize};
//!use serde_json::*;
//!use std::io::{Error, ErrorKind, Read, Write};
//!use std::sync::{Arc, Mutex};
//!use std::fs::{File, OpenOptions};
//!use std::ops::{Deref, DerefMut};
//!use std::path::Path;
//!use std::result::Result;
//!use async_trait::async_trait;
//!use std::any::Any;
//!
//!use crate::fbp::fbp_iidmessage::*;
//!use fbp::fbp_node_context::*;
//!use fbp::fbp_node_trait::*;
//!use fbp::fbp_threadsafe_wrapper::{ThreadSafeType, ThreadSafeOptionType};
//!
//!// This FBP node simply passes incoming IIDMessages to any nodes that
//!// have registered to receive the output of this node.
//!#[derive(Clone, Serialize, Deserialize)]
//!pub struct PassthroughNode {
//!    data: Box<FBPNodeContext>,
//!}
//!
//!impl PassthroughNode {
//!
//!    pub fn new() -> Self {
//!        let mut result = PassthroughNode {
//!            data: Box::new(FBPNodeContext::new("PassthroughNode")),
//!        };
//!
//!        result.node_data().set_node_is_configured(true);
//!        result.clone().start();
//!        result
//!    }
//!}
//!
//!
//!#[async_trait]
//!impl FBPNodeTrait for PassthroughNode {
//!
//!    fn node_data_clone(&self) -> FBPNodeContext {
//!        self.data.deref().clone()
//!    }
//!
//!    fn node_data(&self) -> &FBPNodeContext { &self.data }
//!
//!    fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
//!
//!    fn process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
//!        Ok(msg.clone())
//!    }
//!}
//!
//!// This FBP Node will take an incoming IIDMessage and append data to the
//!// payload of the message and then send it on.
//!#[derive(Clone, Serialize, Deserialize)]
//!pub struct AppendNode {
//!    data: Box<FBPNodeContext>,
//!
//!    #[serde(skip)]
//!    append_data: ThreadSafeOptionType<String>,
//!}
//!
//!impl AppendNode {
//!    pub fn new() -> Self {
//!        let mut result = AppendNode {
//!            data: Box::new(FBPNodeContext::new("AppendNode")),
//!            append_data: ThreadSafeOptionType::new(None),
//!        };
//!
//!        result.clone().start();
//!        result
//!    }
//!
//!    pub fn set_append_data(&mut self, data: String) {
//!        self.append_data.set_option(Some(data));
//!        // This is the only outstanding field that needed to be configured
//!        // once set, the node is configured.
//!        self.data.set_node_is_configured(true);
//!    }
//!}
//!
//!#[async_trait]
//!impl FBPNodeTrait for AppendNode {
//!
//!    fn node_data_clone(&self) -> FBPNodeContext {
//!        self.data.deref().clone()
//!    }
//!
//!    fn node_data(&self) -> &FBPNodeContext { &self.data }
//!
//!    fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
//!
//!    // Here is an example of a node needing additional data before it can start processing
//!    // incoming IIDMessages.  The AppendNode FBP Node needs to be configured with the
//!    // string that will be appended to incoming messages.  That is why the process_config
//!    // method is implemented.  It will parse the incoming Config message and will then call
//!    // the set_append_data method after the string has been extracted from the payload.
//!    fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
//!        if msg.msg_type() == MessageType::Config {
//!            if msg.payload().is_some() {
//!                let payload = msg.payload().as_ref().unwrap();
//!                let config_message: ConfigMessage = serde_json::from_str(&payload)
//!                    .expect("Failed to deserialize the config message");
//!
//!                match config_message.msg_type() {
//!                    ConfigMessageType::Field => {
//!                        if config_message.data().as_ref().is_some() {
//!                            let config_str = json!(config_message.data().as_ref().unwrap());
//!                            let key_str = "append_data";
//!                            if config_str.to_string().contains(key_str) {
//!                                let json_str = config_str.as_str().unwrap();
//!
//!                                let convert_result = serde_json::from_str(json_str);
//!                                if convert_result.is_ok() {
//!                                    let json_value: Value = convert_result.unwrap();
//!                                    let the_value = &json_value[key_str];
//!                                    let append_str = String::from(the_value.as_str().unwrap());
//!
//!                                    self.set_append_data(append_str);
//!                                }
//!                            }
//!                        }
//!                    }
//!                    ConfigMessageType::Connect => {
//!                        // Deal with a Connect
//!                        // This is not implemented for this example
//!                    }
//!                    ConfigMessageType::Disconnect => {
//!                        // Deai with a Disconnect
//!                        // This is not implemented for this example
//!                    }
//!                };
//!            } //  if msg.payload.is_some()
//!        } // if msg.msg_type == MessageType::Config
//!
//!        // Configuration messages should almost never be propagated as they relate to a specific
//!        // FBP node.  Sending an Invalid message will stop message propagation.
//!        Ok(IIDMessage::new(MessageType::Invalid, None))
//!    }
//!
//!    // Given that the AppendNode does some work, it needs to implement the process_message
//!    // method to do that work
//!    fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, Error> {
//!        let string_ref = self.append_data.get_option().as_ref().unwrap().clone();
//!
//!        if msg.payload().is_some() {
//!            let mut payload = msg.payload().as_ref().unwrap().clone();
//!            if self.append_data.is_some() {
//!                payload.push_str(string_ref.as_str());
//!            }
//!
//!            let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
//!            return Ok(new_msg);
//!        } else {
//!            if self.append_data.is_some() {
//!                let new_msg = IIDMessage::new(MessageType::Data, Some(string_ref));
//!                return Ok(new_msg);
//!            }
//!        }
//!
//!        Ok(msg.clone())
//!    }
//!}
//!
//!#[derive(Clone, Serialize, Deserialize)]
//!pub struct LoggerNode {
//!    data: Box<FBPNodeContext>,
//!
//!    #[serde(skip)]
//!    log_file_path: ThreadSafeOptionType<String>,
//!}
//!
//!impl LoggerNode {
//!    pub fn new() -> Self {
//!        let mut result = LoggerNode {
//!            data: Box::new(FBPNodeContext::new("LoggerNode")),
//!            log_file_path: ThreadSafeOptionType::new(None),
//!        };
//!
//!        result.clone().start();
//!        result
//!    }
//!
//!
//!    pub fn set_log_file_path(&mut self, log_file_path: String) {
//!        self.log_file_path.set_option( Some(log_file_path));
//!
//!        // Ensure the File
//!        let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();
//!        let file_path = Path::new(string_ref.as_str());
//!        let file = File::create(file_path).expect("Unable to create file");
//!        drop(file);
//!
//!        self.data.set_node_is_configured(true);
//!    }
//!
//!    pub fn get_log_string(&self) -> Result<String, Error> {
//!        if self.log_file_path.is_none() {
//!            return Err(Error::new(ErrorKind::Other, "Cannot get log string until the node is setup"));
//!        }
//!
//!        let mut contents = String::new();
//!        let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();
//!
//!        let file_path = Path::new(string_ref.as_str());
//!
//!        Ok(contents)
//!    }
//!
//!    pub fn log_string_to_file(&self, data: &String) -> Result<(), Error> {
//!        if self.log_file_path.is_none() {
//!            return Err(Error::new(ErrorKind::Other, "Cannot get log to file until the node is setup"));
//!        }
//!
//!        let string_ref = self.log_file_path.get_option().as_ref().unwrap().clone();
//!        let file_path = Path::new(string_ref.as_str());
//!
//!        let mut file = OpenOptions::new().append(true)
//!            .open(file_path)
//!            .expect("Failed to open file for append");
//!
//!        let string_to_write = data.clone();
//!        let string_to_write = string_to_write.replace("\0", "");
//!
//!        let _write_result = file.write(string_to_write.as_bytes());
//!        Ok(())
//!    }
//!}
//!
//!#[async_trait]
//!impl FBPNodeTrait for LoggerNode {
//!
//!    fn node_data_clone(&self) -> FBPNodeContext {
//!        self.data.deref().clone()
//!    }
//!
//!    fn node_data(&self) -> &FBPNodeContext { &self.data }
//!
//!    fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
//!
//!    // Implement the process_config to use the log file path
//!    fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
//!        if msg.msg_type() == MessageType::Config {
//!            if msg.payload().is_some() {
//!                let payload = msg.payload().as_ref().unwrap();
//!                let config_message: ConfigMessage = serde_json::from_str(&payload)
//!                    .expect("Failed to deserialize the config message");
//!
//!                match config_message.msg_type() {
//!                    ConfigMessageType::Field => {
//!                        if config_message.data().as_ref().is_some() {
//!                            let config_str = json!(config_message.data().as_ref().unwrap());
//!                            let key_str = "log_file_path";
//!                            if config_str.to_string().contains(key_str) {
//!                                let json_str = config_str.as_str().unwrap();
//!                                let convert_result = serde_json::from_str(json_str);
//!                                if convert_result.is_ok() {
//!                                    let json_value: Value = convert_result.unwrap();
//!                                    let the_value = &json_value[key_str];
//!                                    let log_file_path = String::from(the_value.as_str().unwrap());
//!                                    self.set_log_file_path(log_file_path);
//!                                }
//!                            }
//!                        }
//!                    }
//!                    ConfigMessageType::Connect => {
//!                        // Deal with a Connect
//!                        // This is not implemented for this example
//!                    }
//!                    ConfigMessageType::Disconnect => {
//!                        // Deai with a Disconnect
//!                        // This is not implemented for this example
//!                    }
//!                };
//!            } //  if msg.payload.is_some()
//!        } // if msg.msg_type == MessageType::Config
//!
//!        Ok(IIDMessage::new(MessageType::Invalid, None))
//!    }
//!
//!    // Implement the process_message to do the work of this node by writing the log to a file
//!    fn process_message(&mut self, msg: IIDMessage) -> Result<IIDMessage, Error> {
//!        if msg.payload().is_some() {
//!            if self.log_string_to_file(&msg.clone().payload().as_ref().unwrap()).is_err() {
//!                return Err(Error::new(ErrorKind::Other, "Failed to write message to log file"));
//!            }
//!        }
//!
//!        Ok(msg.clone())
//!    }
//!}
//! ```
//!

use async_trait::async_trait;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::{thread, thread::JoinHandle};
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

#[allow(unused_imports)]
#[cfg(not(debug_assertions))]
use log::{error, warn, info, debug, trace};

#[allow(unused_imports)]
#[cfg(debug_assertions)]
use std::{println as error, println as warn, println as info, println as debug, println as trace};


use crate::fbp_iidmessage::*;
use crate::fbp_node_context::*;


pub struct NodeThreadJoin{
    pub name: String,
    jh: Option<std::thread::JoinHandle<bool>>,
    do_join: AtomicBool,
}


impl NodeThreadJoin {
    pub fn new(join_handle: JoinHandle<bool>, name: String) -> Self {
        NodeThreadJoin {
            name,
            jh: Some(join_handle),
            do_join: AtomicBool::new(false),
        }
    }

    pub fn set_do_join(&mut self, flag: bool) {
        *self.do_join.get_mut() = flag;
    }

    pub fn get_do_join(&mut self) -> bool {
        *self.do_join.get_mut()
    }

    pub fn do_the_join(&mut self) {
        if self.get_do_join() {
            // info!("Joining a {} node thread", self.name.clone());
            self.jh.take().map(|jh| jh.join());
            self.jh = None;
        }
    }
}

static mut NODE_JOIN_HANDLES: Option<Arc<Mutex<HashMap<Uuid,  NodeThreadJoin>>>> = None;
static mut NODE_JOIN_HANDLER_INITIALIZED: bool = false;
const NODE_JOIN_THREAD_SLEEP_TIME: u64 = 2;

pub fn set_do_join_for_node_id(node_id: Uuid, flag: bool) {
    unsafe {
        if !NODE_JOIN_HANDLER_INITIALIZED {
            initialize_thread_join_handler();
        }

        if let Some(join_arc) = &mut NODE_JOIN_HANDLES {
            if let Ok(mut join_item) = join_arc.lock() {
                let mut_hm = join_item.deref_mut();
                if let Some(node_thread_join) = mut_hm.get_mut(&node_id) {
                    // info!("Setting the join flag for {}", node_id.clone());
                    node_thread_join.set_do_join(flag);
                }
            }
        }
    }
}

pub fn push_join_handle(node_id: Uuid, jh: std::thread::JoinHandle<bool>, name:String) {
    unsafe {
        if !NODE_JOIN_HANDLER_INITIALIZED {
            initialize_thread_join_handler();
        }

        if let Some(join_arc) = &mut NODE_JOIN_HANDLES {
            if let Ok(mut join_item) = join_arc.lock() {
                let mut_hm = join_item.deref_mut();
                if let Some(_node_op)= mut_hm.get_mut(&node_id.clone()) {
                    let _foo = 42;
                    //Already in the HashMap
                } else {
                    let node_thread_join = NodeThreadJoin::new(jh, name.clone());
                    // info!("Inserting into HashMap node {} id {}", name.clone(), node_id.clone());
                    mut_hm.insert(node_id, node_thread_join);
                }
            }
        }
    }
}

pub fn process_join_handles() {

    unsafe {
        if !NODE_JOIN_HANDLER_INITIALIZED {
            initialize_thread_join_handler();
        }

        // info!("Starting process_join_handles");

        let mut nodes_to_remove: Vec<Uuid> = vec![];

        if let Some(join_arc) = &mut NODE_JOIN_HANDLES {
            if let Ok(mut join_item) = join_arc.lock() {
                let mut_hm = join_item.deref_mut();
                for (key, value) in mut_hm.iter_mut() {
                    // info!("process_join_handles processing node {} id {}", value.name.clone(), key.clone());
                    if value.get_do_join() {
                        // info!("joining and removing node {} id {}", value.name.clone(), key.clone());
                        value.do_the_join();
                        nodes_to_remove.push(*key);
                    }
                }

                for node_id in nodes_to_remove {
                    mut_hm.remove(&node_id);
                }
                /*
                info!("Nodes still in the HashMap");
                for (key, value) in mut_hm.iter_mut() {
                    info!("\tnode {} id {}", value.name.clone(), key.clone());
                }
                */
            }
        }
    }
}
                                

pub fn initialize_thread_join_handler() {
    unsafe {
        if !NODE_JOIN_HANDLER_INITIALIZED {
            NODE_JOIN_HANDLES = Some(Arc::new(Mutex::new(HashMap::new())));
            NODE_JOIN_HANDLER_INITIALIZED = true;
        } else {
            return;
        }
    }
     
    let _ = thread::spawn( move || {
        loop {
            // info!("Top of join handles thread");
            process_join_handles();
            thread::sleep(Duration::from_secs(NODE_JOIN_THREAD_SLEEP_TIME)); 
        }
    });      
}


#[async_trait(?Send)]
pub trait FBPNodeTrait: FBPNodeTraitClone + Sync + Send + 'static {
    /// Return a reference to the FBPNodeContext
    ///
    /// This must be implemented by the struct adhering to the FBPNode trait
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use async_trait::async_trait;
    /// use std::any::Any;
    /// use std::ops::Deref;
    /// use std::io::{Error};
    ///
    /// use fbp::fbp_node_trait::*;
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    /// use fbp::fbp_threadsafe_wrapper::ThreadSafeType;
    ///
    /// #[derive(Clone, Serialize, Deserialize)]
    ///  pub struct ExampleFBPNode {
    ///     data: Box<FBPNodeContext>,
    ///  }
    ///
    /// impl ExampleFBPNode {
    ///     pub fn new() -> Self {
    ///         ExampleFBPNode {
    ///             data: Box::new(FBPNodeContext::new("ExampleFBPNode")),
    ///         }
    ///     }
    /// }
    ///
    /// #[async_trait]
    /// impl FBPNodeTrait for ExampleFBPNode {
    ///
    ///     fn node_data(&self) -> &FBPNodeContext { &self.data }
    ///
    ///     fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
    ///
    ///     fn node_data_clone(&self) -> FBPNodeContext {
    ///         self.data.deref().clone()
    ///     }
    ///
    ///     // This is where an FBP node processes IIDMessages.  In this example
    ///     // No processing is done and the original message is sent along to all of
    ///     // the FBP nodes that have registered to receive the output of this node.
    ///     fn process_message(&mut self,
    ///         msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
    ///         Ok(msg.clone())
    ///     }
    /// }
    /// ```
    ///
    ///

    fn node_data_clone(&self) -> FBPNodeContext;

    fn node_data(&self) -> &FBPNodeContext;

    /// Return a mutable reference to an FBPNodeContext
    ///
    /// This must be implemented by the struct adhering to the FBPNode trait
    /// Please see the example for the node_data method for details
    ///
    fn node_data_mut(&mut self) -> &mut FBPNodeContext;

    /// Provide the processing for this node.
    ///
    /// This is where a specific node does its specific work.  In this example, all that
    /// is done to to forward the incoming IIDMessage onto any nodes that have registered to
    /// receive the output of this node.
    /// Please see the example for the node_data method for details
    ///
    fn process_message(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, Error>;

    // These methods are implemented by the FBPNode trait

    /// Return is an FBP node is fully configured and can process IIDMessages
    ///
    /// This must be implemented by the struct adhering to the FBPNode trait
    /// Please see the example for the node_data method for details
    ///
    fn node_is_configured(&self) -> bool {
        self.node_data().node_is_configured()
    }

    /// Block waiting on node to be configured
    ///
    /// This method will block the caller until the node is fully configured
    ///
    /// # Example
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use async_trait::async_trait;
    /// use std::any::Any;
    /// use std::ops::{Deref, DerefMut};
    /// use std::io::{Error};
    ///
    /// use fbp::fbp_node_trait::*;
    /// use fbp::fbp_node_context::*;
    /// use fbp::fbp_iidmessage::*;
    /// use fbp::fbp_threadsafe_wrapper::ThreadSafeType;
    ///
    /// #[derive(Clone, Serialize, Deserialize)]
    ///  pub struct ExampleFBPNode {
    ///     data: Box<FBPNodeContext>,
    ///  }
    ///
    /// impl ExampleFBPNode {
    ///     pub fn new() -> Self {
    ///         ExampleFBPNode {
    ///             data: Box::new(FBPNodeContext::new("ExampleFBPNode")),
    ///         }
    ///     }
    /// }
    ///
    /// #[async_trait]
    /// impl FBPNodeTrait for ExampleFBPNode {
    ///
    ///     fn node_data(&self) -> &FBPNodeContext { &self.data }
    ///
    ///     fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }
    ///
    ///     fn node_data_clone(&self) -> FBPNodeContext {
    ///         self.data.deref().clone()
    ///     }
    ///
    ///     // This is where an FBP node processes IIDMessages.  In this example
    ///     // No processing is done and the original message is sent along to all of
    ///     // the FBP nodes that have registered to receive the output of this node.
    ///     fn process_message(&mut self,
    ///         msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
    ///         Ok(msg.clone())
    ///     }
    /// }
    ///
    /// let example_node = ExampleFBPNode::new();
    ///
    /// async fn do_wait(node: &ExampleFBPNode) {
    ///     node.wait_for_node_to_be_configured().await;
    /// }
    ///
    /// do_wait(&example_node);
    ///
    /// ```
    async fn wait_for_node_to_be_configured(&self) {
        self.node_data().wait_for_node_to_be_configured().await;
    }

    /// Process an incoming IIDMessage
    ///
    /// When a IIDMessage is sent to an FBP node, it is enqueue onto the input queue of the
    /// node.  The node runs a thread with a loop.  At the top of the loop, the queue is checked
    /// and if there are no items in the queue, then the node thread blocks waiting for an
    /// IIDMessage to be posted to the input queue.  If there is at least one message in the input
    /// queue, then it will be dequeued and will be processed by this method.  If the message is a
    /// Data message, then the process_message method will be called.  If the message is a Process
    /// message then the process_process_message method will be called.  If the message is a
    /// Config message, then the process_config method will be called.  If any errors occur in the
    /// processing of the IIDMessage, then a Error will be returned.
    fn do_process_message(&mut self,
        msg_to_process: IIDMessage,
    ) -> std::result::Result<(), Error> {
        let processed_msg = match msg_to_process.msg_type() {
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
                    Err(Error::new(
                        ErrorKind::Other,
                        "Node received a Data message BEFORE it was configured",
                    ))
                }
            }

            MessageType::Process => self.process_process_message(msg_to_process.clone()),

            MessageType::Config => self.process_config(msg_to_process.clone()),

            MessageType::Stop => Ok(msg_to_process.clone()),

            MessageType::Invalid => Ok(msg_to_process.clone()),
        };

        if let Ok(processed_msg) = processed_msg {
            let msg_to_send = processed_msg;

            if msg_to_send.msg_type() == MessageType::Invalid {
                return Ok(());
            }

            if self
                .node_data()
                .get_num_items_for_receiver_vec(Some("Any".to_string()))
                > 0
            {
                self.node_data()
                    .post_msg_to_group(msg_to_send.clone(), Some("Any".to_string()));
            }

            return Ok(());
        }
        Err(processed_msg.err().unwrap())
    }

    /// Process an IIDMessage that is an Config message
    ///
    /// This method is called from the do_process_message method to handled incoming config message.
    /// If an FBP node needs to receive an Config message to setup its fields so that it has all of
    /// the information needed to process messages, then the node will need to implement this method.
    fn process_config(&mut self, msg: IIDMessage) -> std::result::Result<IIDMessage, Error> {
        Ok(msg.clone())
    }

    /// Process an IIDMessage that is a Process message
    ///
    /// This method is called from the do_process_message method to handled incoming process message.
    /// It is async as it will call the stop method which is async.
    fn process_process_message(&mut self,
        msg: IIDMessage,
    ) -> std::result::Result<IIDMessage, Error> {
        if msg.payload().is_some() {
            let payload = msg.clone().payload().as_ref().unwrap().clone();

            let process_msg: ProcessMessage =
                ProcessMessage::make_self_from_string(payload.clone().as_str());

            match process_msg.msg() {
                ProcessMessageType::Stop => {
                    // self.stop().await;
                    self.stop();
                }
                ProcessMessageType::Suspend => {
                    if process_msg.message_node().is_some() {
                        if process_msg.message_node().unwrap() == self.node_data().uuid() {
                            self.node_data_mut().set_is_suspended(true);
                        }
                    } else {
                        // I THINK this needs to be specific
                        // self.node_data_mut().set_is_suspended(true);
                    }
                }
                ProcessMessageType::Restart => {
                    if process_msg.message_node().is_some() {
                        if process_msg.message_node().unwrap() == self.node_data().uuid()  && self.node_data().node_is_suspended() {
                            self.node_data_mut().set_is_suspended(false);
                        }
                    } else if self.node_data().node_is_suspended() {
                        self.node_data_mut().set_is_suspended(false);
                    }
                }
            }

            if process_msg.propagate() {
                return Ok(msg.clone());
            }
        }
        // Sending an invalid message so that it will NOT be propagated.
        Ok(IIDMessage::new(MessageType::Invalid, None))
    }

    /// Run the message loop for an FBP node.
    ///
    /// The start method runs a thread that will block waiting on IIDMessages to be enqueued on to
    /// the input of the FBP node.  Once enqueued the loop will dequeue a IIDMessage and then
    /// call do_process_message to determine what the correct processing is needed for the IIDMessage
    ///
    fn start(mut self)
    where
        Self: std::marker::Sized + Send + Sync + Clone + 'static,
    {
        initialize_thread_join_handler();

        if self.node_data().node_is_running() {
            return;
        }

        let node_uuid = self.node_data().uuid();
        let node_name = self.node_data().name();

        // Mark the node as running
        self.node_data().set_node_is_running(true);

        // The FBPNodeContext is in a Box so a clone will return a reference
        // to the same underlying data.
        let node_data: FBPNodeContext = self.node_data_clone();

        let jh = thread::spawn(move || {
            while node_data.node_is_running() {
                // Calling recv() will block this thread if there are no IIDMessages to
                // process.  If there is an IIDMessage the recv will return the IIDMessage
                let msg_to_process = node_data.rx().recv();


                if let Ok(msg_to_process) = msg_to_process {
                    let a_msg = msg_to_process;
                    if a_msg.msg_type() == MessageType::Stop {
                        break;
                    }
                    // Call do_process_message to process the IIDMessage
                    let msg_processing_result = self.do_process_message(a_msg.clone());

                    if msg_processing_result.is_err() {
                        // TODO: Log error to file.
                        /*
                        info!(
                            "Error processing message {}",
                            msg_processing_result.err().unwrap()
                        )
                        */
                    }
                }
            }

            // The while loop has completed.  This means that the node has 'stoppedc'
            // update the node context
            node_data.set_node_has_completed(true);
            true
        });

        // info!("End of thread loop for node {} id {}", node_name.clone(), node_uuid.clone());
        push_join_handle(node_uuid, jh, node_name.clone());

    }

    /// Tell the FBP Node to stop its processing loop
    ///
    /// All good things come to an end.  When a node receives a Process IIDMessage with the stop
    /// message, this method will be called and it will stop running the nodes processing loop
    ///
    fn stop(&self) {

        let the_node_data = self.node_data_clone();

        let _ = thread::spawn(move || {

             //info!("Entering Base stop for {}", the_node_data.name());
            let node_id = the_node_data.uuid();
            // info!("Calling stop on node {} id {}", the_node_data.name(), node_id.clone());
            // Posting a message to ensure that the Stop message is seen which will break out of the thread.
            let msg = IIDMessage::new(MessageType::Stop, None);
            the_node_data.post_msg(msg.clone());
            
            // Give the post above a second to get to the receiver.
            // thread::sleep(Duration::from_secs(1));
            
            let mut keep_waiting = true;
            let mut pushed_message = false;
            while keep_waiting {
                if let Ok(msg) = the_node_data.rx().try_recv() {
                    if msg.msg_type() != MessageType::Stop {
                        // info!("Oops there are still items in the queue for node {}", the_node_data.name());
                        // While I am putting the Message back, I can only  do so at the end.  Given that a Stop has been
                        // posted, this may never get seen.  TODO: figure out how to push to the front of the queue.
                        the_node_data.tx().send(msg.clone());
                        pushed_message = true;
                        // thread::sleep(Duration::from_secs(1));
                    } else {
                    
                        if pushed_message {
                            // bummer
                            // The only thing to do here is to ignore this mmessage and re-push the stop message and  hope the the order will
                            // get right at some point
                            // info!("We push an IID behind the stop message.  Eating the message and re posting");
                            the_node_data.post_msg(msg.clone());
                            pushed_message = false;
                            continue;
                        }
                        // The message in the qu
                        // info!("Only Stop Node {} id {} is set to be joined", the_node_data.name(), node_id.clone());
                        
                        the_node_data.set_node_is_running(false);
                    
                        // Tell the thread join infrastructure that this node thread can be joined.
                        set_do_join_for_node_id(node_id, true);
                        // info!("Node {} id {} is set to be joined", the_node_data.name(), node_id.clone());
                        keep_waiting = false;
                    }
                } else {
                    // we have come to the end of the line
                    // info!("node {} has all of its messages drained,  Stopping", the_node_data.name());
                    // Since the message above will break out of the thread loop, ensure that the node is marked as not running
                    the_node_data.set_node_is_running(false);
                
                    // Tell the thread join infrastructure that this node thread can be joined.
                    set_do_join_for_node_id(node_id, true);
                    // info!("Node {} id {} is set to be joined", the_node_data.name(), node_id.clone());
                    keep_waiting = false;
                }
            }
        
            // info!("Leaving Base stop for {}", the_node_data.name());

        }).join();
    }

}

pub trait FBPNodeTraitClone {
    fn clone_box(&self) -> Box<dyn FBPNodeTrait>;
}

impl<T> FBPNodeTraitClone for T
where
    T: 'static + FBPNodeTrait + Clone,
{
    fn clone_box(&self) -> Box<dyn FBPNodeTrait> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FBPNodeTrait> {
    fn clone(&self) -> Box<dyn FBPNodeTrait> {
        self.clone_box()
    }
}
