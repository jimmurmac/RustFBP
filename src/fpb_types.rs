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
// struct FPBNode_struct
// 
// Define the basic structure of a FPBNode. Every Node should have one of 
// these as it's data members
// ----------------------------------------------------------------------------

struct FPBNode_struct {
    name: Option<&' static str>,
    uuid: Uuid,
    sender: std::sync::mpsc::Sender<IIDMessage>,
    receiver: std::sync::mpsc::Receiver<IIDMessage>,
    state: NodeState,
    node_thread: Option<std::thread::Thread>,
}

 impl FPBNode_struct {
     //Constructor
     pub fn new() -> Self {
         let (s, r) = channel::<IIDMessage>();
         FPBNode {
             sender: s,
             receiver: r,
             state: NodeState::Quiescent,
             node_thread: None,
         }
     }
 }

// ----------------------------------------------------------------------------
// trait FPBNodeTrait
// 
// Define a trait (interface) for a FPBNode.  These methods provide the means
// for interacting with a node
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
    fn new(name: Option<&'static str>) -> Self;
    fn members(&self) -> &mut FPBNode_struct;

    fn name(&self) -> Option<&' static str>; {
        self.members().unwrap().name
    }

    fn uuid(&self) -> &Uuid {
        self.members().unwrap().uuid
    }

    fn node_state(&self) -> NodeState {
        self.members().unwrap().state
    }

    fn input_port(&self) -> &std::sync::mpsc::Receiver<IIDMessage> {
        self.members().unwrap().sender
    }

    fn post_message(&self, msg: IIDMessage) {
        self.input_port().send(msg)
    }

    fn add_receiver(&self, receiver: mpsc::Receiver<IIDMessage>);
    fn start(&self);
    fn stop(&stop);
    
    fn process_data(&self);
}

 
// ----------------------------------------------------------------------------
// struct FPBNode_Echo
// 
// Define a FPBNode that will echo it input to the console
// ----------------------------------------------------------------------------

struct FPB_Echo {
    members: FPBNode_struct;
}

impl FPBNodeTrait for FPB_Echo








#[cfg(test)]
mod test {
    use super::IIDMessage;
    use super::FPBNode;
    use super::NodeState;
    use super::MessageType;
    use super::AppendPayload;

    #[test]
    fn type_test() {
        let msg = IIDMessage::new(MessageType::Data, None);
        assert_eq!(msg.msg_type(), MessageType::Data);
        assert_eq!(msg.payload.is_none(), true);

        let node = FPBNode::new();
        assert_eq!(node.node_thread.is_none(), true);
    }

    #[test]
    fn payload_test() {
        let payload = AppendPayload { front: "Front".to_string(), end: "End".to_string()};
        let serialized = serde_json::to_string(&payload).unwrap();
        let msg = IIDMessage::new(MessageType::Data, Some(serialized.clone()));
        assert_eq!(msg.msg_type(), MessageType::Data);
        assert_eq!(msg.payload().unwrap().as_str(), serialized.as_str());

        let node  = FPBNode::new();
        assert_eq!(node.node_thread.is_none(), true);
        assert_eq!(node.state, NodeState::Quiescent);
    }

}




