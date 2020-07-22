use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;
use std::thread;
use std::string::String;
use uuid::Uuid;
use std::thread::JoinHandle;
use std::sync::Arc;
use std::sync::Mutex;
use std::ops::DerefMut;
use std::borrow::Borrow;

// ----------------------------------------------------------------------------
// enum MessageType
//
// Provide a way to differentiate different messages sent for processing
// ----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MessageType {
    Data,       // This is a 'normal' data packet to be processed by a node
    Config,     // This is a configuration packet used to configure a node
}

// ----------------------------------------------------------------------------
// struct IIDMessage<T>
//
// Define the standard data packet that will be sent to a node.  Curently it is
// defined as a generic.  This would allow for different payload types.
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct IIDMessage {
    msg_type: MessageType,
    payload: Option<String>,
}

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


#[derive(Debug, Clone)]
pub struct FPBNodeStruct {
    name: Option<&'static str>,
    uuid: Uuid,
    tx: Option<Arc<Mutex<Sender<IIDMessage>>>>,
    rx: Option<Arc<Mutex<Receiver<IIDMessage>>>>,
    node_join_handle: Option<Arc<Mutex<JoinHandle<()>>>>,
    output_vec: Option<Arc<Mutex<Vec<Sender<IIDMessage>>>>>
}

impl FPBNodeStruct {
    //Constructor
    fn new(name: Option<&'static str>) -> Self {
        FPBNodeStruct {
            name: name,
            uuid: Uuid::new_v4(),
            tx: None,
            rx: None,
            node_join_handle: None,
            output_vec: None,
        }
    }

    pub fn node_uuid(&self) -> Uuid {
        self.uuid.clone()
    }
}

// ----------------------------------------------------------------------------
// trait FPBNodeTrait
//
// Define a trait (interface) for a FPBNode.  These methods provide the means
// for interacting with a node
//
//  members:
//          To facilitate sharing in a composition system like Rust, a specific
//          implementation of a FPBNode will 'have a' boxed (heap based)
//          FPBNodeStruct member.  This allows for sharing the basic member
//          variable structure that is necessary for every Node.  All of the
//          other member accessors are based upon getting this FPBNodeStruct
//          and using that to obtain the specific variable
//
//  name:   Each node should have a name.  The field is an Option so that it
//          may be None
//
//  uuid:   Each node will have a unique ID (uuid).  This can be used to
//          configure a network of nodes
//
//  node_state:
//          Returns the current state of the node.
//
//  input_channel:
//          Returns the input channel for the node.
//
//  output_channel:
//          Returns the outut channel for the node.
//
//  post_message:
//          A Node acts like an independant processing unit.  It only works on
//          the messages (IIDMessage) that are sent to it.  The post_message
//          method takes a message and puts it into a mpsc input channel to be
//          processed.
//
//  add_receiver:
//          A Node will output it's data to a vector of other nodes.  NOTE:
//          the default for a new node will be to have it's vector of receivers
//          be empty which is the same as sending to /dev/null
//
//  start:  Each Node is a aynchronous processing node.  It runs on a thread.
//          if the node is quiescent, then a thread will be created and that
//          thread will call the process_data method.  This will continue until
//          the node is stopped.
//
//  stop:   Once a Node has started, this method will stop the processing thread
//          which will stop all processing for this Node.
//
//  process_data:
//          This is the 'heart and soul' of a Node.  When start is called upon
//          a Node, the process_data method will pull messages and process them
//          as needed to fullfil the role of this node.
//
// ----------------------------------------------------------------------------
trait FPBNodeTrait {

    fn node_struct(&self) -> FPBNodeStruct;

    fn have_tx(&self)-> bool  {
        self.node_struct().tx.is_some()
    }

    fn sender(&self) -> Option<Arc<Mutex<Sender<IIDMessage>>>> {
        self.node_struct().tx
    }

    fn receiver(&self) -> Option<Arc<Mutex<Receiver<IIDMessage>>>> {
        self.node_struct().rx
    }

    fn join_handle(&self) -> Option<Arc<Mutex<JoinHandle<()>>>> {
        self.node_struct().node_join_handle
    }

    fn output_vec(&self) -> Option<Arc<Mutex<Vec<Sender<IIDMessage>>>>> {
        self.node_struct().output_vec
    }

    fn process_message(&self, msg: Option<IIDMessage>) -> Option<IIDMessage> {
        if msg.is_some() {
            return Some(msg.unwrap().clone());
        }
        None
    }

    fn do_start(self: &'static Self) -> Result<JoinHandle<()>, &'static str>
    //fn do_start(self: &'static Self, st: &'static FPBnodeStruct) -> Result<JoinHandle<()>, &'static str>
        where Self: std::marker::Sized + std::marker::Sync + std::marker::Send {


        //fn do_start(&self,//rx: Option<Arc<Mutex<Receiver<IIDMessage>>>>,
        //ov: Option<Arc<Mutex<Vec<Sender<IIDMessage>>>>>,
        //this: &'static dyn RFPBNode) -> Result<JoinHandle<()>, &'static str> {
        // If rx is none, then bail.

        //let local_self = *this;
        //if self.receiver().is_none()  {
        if self.node_struct().rx.is_none() {
            return  Err("rx is None")
        }

        // Spawn a thread that will loop until the node is stopped.
        // Note the call to recv will block this thread if there are
        // no messages to be processed.
        let joiner = thread::spawn(move || {
            // Clone the input each time through the loop.
            // This is required to keep the ownership happy.
            //let rxc = self.node_struct().rx.clone();
            let rxc = self.borrow().node_struct().rx.clone();

            // While there is a Receiver, pull messages and dispatch them

            while rxc.is_some() {
                let rxcc = rxc.clone();
                let rxccu  = rxcc.unwrap();
                let rxccul = rxccu.lock();
                let mut rxcculu = rxccul.unwrap();
                let rxcculur = rxcculu.deref_mut();
                let msg = rxcculur.recv().unwrap();
                //let msg = rxc.clone().unwrap().clone().lock().unwrap().deref_mut().recv().unwrap();
                //let sc = self.node_struct().output_vec.clone();
                let sc = self.borrow().node_struct().output_vec.clone();
                let sca = sc.unwrap().clone();
                let mut scmg = sca.lock().unwrap();
                let senders = scmg.deref_mut();
                for sender in senders {
                    let rmsg = self.process_message(Some(msg.clone()));
                    if rmsg.is_some() {
                        let _ = sender.send(rmsg.unwrap());
                    }
                    // Some error message should be sent here.
                }
            }
        });

        return Ok(joiner);
    }

    fn start(&'static mut self)  where Self: std::marker::Sized + std::marker::Sync + std::marker::Send {

        if self.have_tx() {
            return
        }

        let (tx, rx) = channel::<IIDMessage>();
        let a_vec: Vec<Sender<IIDMessage>> = Vec::new();

        self.node_struct().tx = Some(Arc::new(Mutex::new(tx)));
        self.node_struct().rx = Some(Arc::new(Mutex::new(rx)));
        self.node_struct().output_vec = Some(Arc::new(Mutex::new(a_vec)));
        let join_result = Self::do_start(self);

        self.node_struct().node_join_handle = Some(Arc::new(Mutex::new(join_result.unwrap())));
    }
}


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




