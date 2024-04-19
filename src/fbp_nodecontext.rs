use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{self, Visitor, SeqAccess, MapAccess};
use serde::ser::{SerializeStruct};
use std::sync::atomic::{Ordering, AtomicBool};
use uuid::Uuid;
use std::fmt;
use std::ops::{Deref};
use std::collections::HashMap;

use crate::fbp_asyncstate::*;
use crate::fbp_receivercontext::*;
use crate::fbp_iidmessage::*;

/*
#[derive(Debug, Clone)]
pub enum Nodes {
    PassthroughNode(Option<Arc<PassthroughNode>>),
    AppendNode(Option<Arc<AppendNode>>),
    LoggerNode(Option<Arc<LoggerNode>>),
    // ADD NEW NODE HERE
    Invalid,
}
*/

/* --------------------------------------------------------------------------
    struct FBPNodeContext

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

        is_configured:
                    An AsyncState that can be used to see if this node is 
                    configured.

        is_running:
                   An AsyncState that can be used to see if this node is 
                    running.

        node_completion:
                    An AsyncState that can be used to see if this node has
                    completed processing (stopped)
   -------------------------------------------------------------------------- */             

   #[derive(Debug, Clone)]
   pub struct FBPNodeContext { 
       pub name: String,
       pub uuid: Uuid,
       pub tx: Arc<Mutex<Sender<IIDMessage>>>,
       pub rx: Arc<Mutex<Receiver<IIDMessage>>>,
       pub output_vec: Arc<Mutex<HashMap<String, Vec<Arc<Mutex<ReceiverContext>>>>>>,
       pub is_configured: AsyncState,
       pub is_running: AsyncState,
       pub node_completion: AsyncState,
       pub node_suspended: Arc<AtomicBool>,
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
                               
           wait_for_node_to_running:
               Description:    This is an async/await method that will wait 
                               until this node is running.
   
           node_has_completed:
               Description:    Returns true if the node has completed (stopped)
   
           set_node_is_configured:
               Parameters:
                   flag        A boolean that specifies if the node as completed 
                               or not
   
           wait_for_node_to_complete: 
               Description:    This is an async/await method that will wait until
                               this node has completed (stoppec).alloc
   
           node_is_configured: 
               Description:    Returns true if the node has been configured
   
           set_node_is_configured: 
               Parameters: 
                   flag        A Boolean that specifies if the node is configured.
   
           wait_for_node_to_be_configured: 
               Description:    This is an async/await method that will wait until
                               this node has been configured.
   
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
               output_vec: Arc::new(Mutex::new(HashMap::new())),
               is_configured: AsyncState::new(),
               is_running: AsyncState::new(),
               node_completion: AsyncState::new(),
               node_suspended: Arc::new(AtomicBool::new(false)),
           }
       }
   
       // These functions allow for testing if a node is running,
       // setting the node to the running state and waiting on
       // the node to be running
       pub fn node_is_running(&self) -> bool {
           self.is_running.is_ready()
       }
   
       pub fn set_node_is_running(&self, flag: bool) {
           self.is_running.set_is_ready(flag);
       }
   
       pub async fn wait_for_node_to_running(&self) {
           self.is_running.clone().await;
       }
   
       // These functions allow for testing if a node has left it's,
       // processing loop, setting the state on processing and waiting
       // for the processing loop to end
       pub fn node_has_completed(&self) -> bool {
           self.node_completion.is_ready()
       }
   
       pub fn set_node_has_completed(&self, flag: bool) {
           self.node_completion.set_is_ready(flag);
       }
   
       pub async fn wait_for_node_to_complete(&self) {
           self.node_completion.clone().await;
       }
   
       pub fn node_is_configured(&self) -> bool {
           self.is_configured.is_ready()
       }
   
       pub fn set_node_is_configured(&self, flag: bool) {
           self.is_configured.set_is_ready(flag);
       }
   
       pub async fn wait_for_node_to_be_configured(&self) {
           self.is_configured.clone().await;
       }
   
       pub fn node_is_suspended(&self) -> bool {
           self.node_suspended.deref().load(Ordering::Relaxed)
       }
   
       pub fn set_is_suspended(&self, flag: bool) {
           self.node_suspended.store(flag, Ordering::Relaxed)
       }
   
       pub fn add_receiver(&mut self, receiver: &mut FBPNodeContext, key: Option<String>) {
           let mut hash_key = "Any".to_string();

           if key.is_some() {
               hash_key = key.clone().unwrap();
           }

           let new_receiver = ReceiverContext::new(receiver);

           if  self.output_vec.lock().unwrap().get_mut(&hash_key).is_some() {
               self.output_vec.lock().unwrap().get_mut(&hash_key).unwrap().push(Arc::new(Mutex::new(new_receiver)));
           } else {
               let mut vec_for_key: Vec<Arc<Mutex<ReceiverContext>>> = Vec::new() ;
               vec_for_key.push(Arc::new(Mutex::new(new_receiver)));
               self.output_vec.lock().unwrap().insert(hash_key, vec_for_key);
           }
       }
   
       pub fn remove_receiver(&mut self, receiver: &mut FBPNodeContext, key: Option<String>) {

           let mut hash_key = "Any".to_string();

           if key.is_some() {
               hash_key = key.clone().unwrap();
           }

           if  self.output_vec.lock().unwrap().get_mut(&hash_key).is_some() {
               let rr = ReceiverContext::new(receiver);
               let index = self.output_vec.lock().unwrap().get_mut(&hash_key).unwrap().iter().position(|r| r.lock().unwrap().deref() == &rr).unwrap();
               self.output_vec.lock().unwrap().get_mut(&hash_key).unwrap().remove(index);
           }
       }


       pub fn get_num_items_for_receiver_vec(&mut self, key: Option<String>) -> usize {
           let mut hash_key = "Any".to_string();

           if key.is_some() {
               hash_key = key.clone().unwrap();
           }

           let mut result: usize = 0;
           if  self.output_vec.clone().lock().unwrap().get(&hash_key).is_some() {
                if self.output_vec.clone().lock().unwrap().get(&hash_key).is_some() {
                    result = self.output_vec.clone().lock().unwrap().get(&hash_key).unwrap().len();
                }
           }

           result
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
   
           enum Field { Name/*, Tx, Rx, Output_vec, Is_running */}
   
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
   
   impl PartialEq for FBPNodeContext {
       fn eq(&self, other: &Self) -> bool {
           self.uuid == other.uuid
       }
   }
