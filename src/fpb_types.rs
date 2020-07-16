use std::sync::mpsc::channel;
use uuid::Uuid;

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
// enum NodeState
// 
// Enumerate the current state of a Node
// ----------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeState {
    Quiescent,
    Running,
}

// ----------------------------------------------------------------------------
// struct IIDMessage<T>
// 
// Define the standard data packet that will be sent to a node.  Curently it is
// defined as a generic.  This would allow for different payload types.
// ----------------------------------------------------------------------------

//#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn payload(&self) -> Option<String> {
        if self.payload.is_none() {
            return None
        }
        else
        {
           Some(self.payload.as_ref().unwrap().clone())
        }
    }
}

// ----------------------------------------------------------------------------
// struct FPBNodeStruct
// 
// Define the basic structure of a FPBNode. Every Node should have one of 
// these as it's data members
// ----------------------------------------------------------------------------

struct FPBNodeStruct {
    name: Option<&'static str>,
    uuid: Uuid,
    sender: std::sync::mpsc::Sender<IIDMessage>,
    receiver: std::sync::mpsc::Receiver<IIDMessage>,
    state: NodeState,
    node_thread: Option<std::thread::Thread>
}

 impl FPBNodeStruct {
     //Constructor
     pub fn new(name: Option<&'static str>) -> Self {
         let (s, r) = channel::<IIDMessage>();
         FPBNodeStruct {
             name: name,
             uuid: Uuid::new_v4(),
             sender: s,
             receiver: r,
             state: NodeState::Quiescent,
             node_thread: None
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
trait FPBNodeTrait {

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

    // Accessor to get the current state of the node.
    // NOTE: This might need to be made atomic
    fn node_state(&mut self) -> NodeState {
        self.members().state
    }

    // Accessor to get the sender channel
    fn input_channel(&mut self) -> std::sync::mpsc::Sender<IIDMessage> {
        self.members().sender
    }

    fn output_channel(&mut self) -> std::sync::mpsc::Receiver<IIDMessage> {
        self.members().receiver
    }

    // Post a message to the input queue
    fn post_message(&mut self, msg: IIDMessage) {
        let _ = self.members().sender.send(msg);
        return
    }

    // Add an output node that will receive the output of 
    // this node.  NOTE: This will need to be implemented as 
    // a dynamic vector.
    fn add_receiver(&mut self, receiver: std::sync::mpsc::Receiver<IIDMessage>);

    // Start the node to beginning processing.  If the node has already
    // been started then this method will do nothing.
    fn start(&mut self);

    // Stop all processing for this node.  It remains to be seen 
    // if this function will block until all currently enqueued message
    // will be processed, or all unprocessed message will be ignored.
    fn stop(&mut self);
    
    // This is the processing loop for this Node.
    fn process_data(&mut self);
   
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




