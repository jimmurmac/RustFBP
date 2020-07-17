use std::sync::mpsc::channel;
use uuid::Uuid;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::sync::Arc;
use std::borrow::Borrow;
use std::sync::Mutex;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IIDMessage {
    msg_type: MessageType,
    payload: Option<&'static str>,
}

impl IIDMessage {
    // Constructor for a IIDMessage
    pub fn new(msg_type: MessageType, payload: Option<String>) -> Self {
        IIDMessage {
            msg_type: msg_type,
            payload: Box::new(payload),
        }
    }

    // Return the message type for a IIDMessage
    pub fn msg_type(&self) -> MessageType {
        self.msg_type
    }

    // Return a reference to the payload of a IIDMessage
    pub fn payload(&self) -> &Option<String> {
        if self.payload.is_none() {
            return &None;
        }
        else
        {
            self.payload.as_ref()
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct FPBNodeStruct<'a, 'b> {
    name: Option<&'static str>,
    uuid: Uuid,
    tx: &Box<std::sync::mpsc::Sender<IIDMessage>>,
    rx: &Box<std::sync::mpsc::Receiver<IIDMessage>>,
    state: AtomicBool,
    node_thread: Option<std::thread::JoinHandle<()>>,
    output_vec: std::vec::Vec<std::sync::mpsc::Sender<Box<IIDMessage>>>
}

impl FPBNodeStruct {
    //Constructor
    pub fn new(name: Option<&'static str>) -> Self {
        let (tx, rx) = channel::<Box<IIDMessage>>();
        FPBNodeStruct {
            name: name,
            uuid: Uuid::new_v4(),
            tx: Box::new(tx),
            rx: Box::new(rx),
            state: AtomicBool::new(false),
            node_thread: None,
            output_vec: Vec::<std::sync::mpsc::Sender<Box<IIDMessage>>>::new()
        }
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
trait FPBNodeTrait<'a> {

    // Constructor
    //fn new(name: Option<&'static str>) -> Self;

    // Accessor for the boxed FPBNodeStruct member variable
    fn members(&mut self) -> Box<FPBNodeStruct>;

    // Accessor to get the name of a Node
    fn name(&mut self) -> Option<&'static str> {
        self.members().name
    }

    // Accessor to get the uuid of a Node
    fn uuid(&mut self) -> Uuid {
        self.members().uuid
    }

    fn is_running(&mut self) -> bool {
        self.members().state.load(Ordering::Relaxed) == true
    }

    fn output_vector(&mut self) -> std::vec::Vec<std::sync::mpsc::Sender<Box<IIDMessage>>> {
        self.members().output_vec
    }

    // Post a message to the input queue
    fn post_message(&mut self, msg: Box<IIDMessage>) {
        let _ = self.members().tx.send(msg);
        return
    }

    // Add an output node that will receive the output of
    // this node.
    fn add_receiver(&mut self, sender: std::sync::mpsc::Sender<Box<IIDMessage>>) {
        self.members().output_vec.push(sender)
    }

    // This is the processing loop for this Node. It should be re-implemented
    // for each specific Node. The default processing is to simply write out
    // the input to the vector of output receivers.
    fn process_data(&mut self) {
        let msg_result = self.members().rx.recv();
        if msg_result.is_err() {
            return
        }

        let msg = msg_result.unwrap();

        for sender in self.members().output_vec {
            let msg_to_process = msg.clone();
            sender.send(msg_to_process);
        }
    }

    fn start(mut self) where Self: 'static + std::marker::Sized + std::marker::Send {
        if self.is_running() {
            return
        }
        let handler = thread::spawn(move || {
            let mut loop_condition = true;
            while loop_condition {
                self.process_data();
                loop_condition = self.is_running();
            }
        });

        self.members().node_thread = Some(handler);
    }

    // Stop all processing for this node.  It remains to be seen
    // if this function will block until all currently enqueued message
    // will be processed, or all unprocessed message will be ignored.
    fn stop(&mut self) {
        if self.is_running() {
            *self.members().state.get_mut() = false;
            self.members().node_thread.unwrap().join();
            self.members().node_thread = None;
        }
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




