use std::sync::mpsc::channel;

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

pub struct IIDMessage<T> {
    msg_type: MessageType,
    payload: Option<T>,
}

impl<T> IIDMessage<T> {
    // Constructor for a IIDMessage
    pub fn new(msg_type: MessageType, payload: Option<T>) -> Self {
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
    pub fn payload(&self) -> &Option<T> {
        &self.payload
    }
}

// ----------------------------------------------------------------------------
// struct FPBNode
// 
// Define the basic structure of a FPBNode.  It is defined as a generic so that
// different IIDMessages can be used
// ----------------------------------------------------------------------------

struct FPBNode<T> {
    sender: std::sync::mpsc::Sender<IIDMessage<T>>,
    receiver: std::sync::mpsc::Receiver<IIDMessage<T>>,
    state: NodeState,
    node_thread: Option<std::thread::Thread>,
}

impl<T> FPBNode<T> {
    //Constructor
    pub fn new() -> Self {
        let (s, r) = channel::<IIDMessage<T>>();
        FPBNode {
            sender: s,
            receiver: r,
            state: NodeState::Quiescent,
            node_thread: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::IIDMessage;
    use super::FPBNode;
    use super::NodeState;
    use super::MessageType;

    fn type_test() {
        let msg = IIDMessage::new( MessageType::Data, None);
        assert_eq!(msg.msg_type(), MessageType::Data);
        assert_eq!(msg.payload(), None);

        let node = FPBNode::new();
        assert_eq!(node.node_thread, None);
        assert_eq!(node.state, NodeState::Quiescent);
    }
}




