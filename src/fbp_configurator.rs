/* ==========================================================================
    File:           fbp_configurator.rs

    Description:    This file provides the means for creating a FBP network
                    from a JSON representation.  
                    
                    The current implementation is NOT efficient as it 
                    requires that for every new Node type an enum and a 
                    function with a match statement be updated for the new
                    node. This method was chosen as an expedeancy but a more
                    efficient shared implementation needs to be developed.

    History:        Jim Murphy 09/19/2020   Initial Code.
                    Jim Murphy 09/22/2020   Rewrite after prototype

    Copyright ©  2020 Pesa Switching Systems Inc. All rights reserved.
   ========================================================================== */

use crate::fbp_types::*;
use crate::fbp_test_nodes::*; 
use serde_json::Value;
use std::collections::HashMap;

/* ----------------------------------------------------------------------
    A helper function to get a value for a key value pair in a JSON
    string
    get_value_from_json_map_key:
   ---------------------------------------------------------------------- */
pub fn get_value_from_json_map_key(json_str: &str, key: &str) -> String {
    // Sanitize the json_str just in case
    let clean_string = json_str.replace("\\\"", "");

    let json_value: Value = serde_json::from_str(clean_string.as_str())
        .expect("Could not turn the string into a JSON value");

    let value_value = &json_value[key];
    String::from(value_value.as_str().unwrap())
}
 
/* --------------------------------------------------------------------------
    enum Nodes

    The Nodes enum provides a 'type' that can be returned for every node. 
    With Rust, the return type of a function is part of the type signature 
    of the function.  Couple that with VERY strong typing of Rust, there 
    needed to be a way to have a type that could return back any FBP node.

    One of the features of Rust enumerations is that an enumeration can be 
    associated with a value.  So the Nodes enumeration will have an entry
    for every Node type that can be creates and it will be associated with
    an optional instance of that node.

    To get a value from an enumeration there are two main techniques. 
        1) Use a match statement (see NodeMap::get_node for an example)
        2) Use if let (see Configurator::create_connection for an example)

    Unfortunately this design requires that ALL nodes MUST be listed in the
    Nodes enum for the Configurator to function.

    To help with adding the necessary code a comment with the text

    // ADD NEW NODE HERE 

    will define where the new Node will need to be added.
   -------------------------------------------------------------------------- */
#[derive(Debug, Clone)]
pub enum Nodes {
    PassthroughNode(Option<PassthroughNode>),
    AppendNode(Option<AppendNode>),
    LoggerNode(Option<LoggerNode>),
    // ADD NEW NODE HERE
    Invalid,
}

/* --------------------------------------------------------------------------
    trait NodeConstructor

    Description:    The NodeConstructor trait provides the means for a FBP
                    node to register itself with the "Configurator" so that
                    the node and be created from a JSON string

    Methods:
        register_node:

            Parameters:
                node_name:  The name of the FBP Node Type.  This will become
                            the 'key' when looking up the serialized node.

                serialized_self:
                            The JSON string that when deserialized, will 
                            produce a new instance of the FBP Node

                node_vec:   A mutable reference to a Vector that will hold the name
                            (Key) and the JSON string (value) 
                        

            Description:    This method will 'register' a FBP node with the "configurator"
                            so that when the times comes to create a network of nodes that
                            the registered node can be created.

            Implementation:
                            This method is fully implemented by this trait and a FBP node 
                            does NOT need to implement this method.


        make_node:

            Parameters:
                json_str:   This is the serialized string for a node.  

            Description:    This method will create a FBP node and will 
                            return it as a Nodes enum.  An enum was used 
                            because it is really the only way to return 
                            different FBP nodes using this trait method.

             Implementation:
                            This method must be implemented for every FBP 
                            Node will need to implement this method.

   -------------------------------------------------------------------------- */
pub trait NodeConstructor {

    fn register_node(&self, node_name: &String, 
        serialized_self: Result<String, serde_json::Error>, 
        node_vec: &mut Vec<(String,String)>) {
        if serialized_self.is_ok() {
            let json_str  = serialized_self.unwrap();
            node_vec.push((node_name.clone(), json_str.clone()));
        }
    }

    fn make_node(json_str: &String)-> Nodes;
}

/* --------------------------------------------------------------------------
    struct NodeMap

    Define the standard data struct that will be serialized into a JSON string
    and sent in an IIDMessage struct.  

    Fields:
        msg:        This is a ConfigMessageType.  It defines what type of 
                    configuration message this is.

        data:       This is an optional data string (JSON) of the data for
                    to used to configure a node
   -------------------------------------------------------------------------- */
#[derive(Debug, Clone)]
pub struct NodeMap {
    node_map: HashMap<String, String>,
}

impl NodeMap {

    fn get_node_constructor_vector() -> Vec<(String, String)> {

        let mut node_vec: Vec<(String,String)> = Vec::new();

        // Add the PassthroughNode
        let my_passthrough = PassthroughNode::new();
        my_passthrough.register_node(&my_passthrough.node_data().name, serde_json::to_string(&my_passthrough), &mut node_vec);

        // Add the AppendNode
        let my_append = AppendNode::new();
        my_append.register_node(&my_append.node_data().name, serde_json::to_string(&my_append), &mut node_vec);

        // Add the LoggerNode
        let my_logger = LoggerNode::new();
        my_logger.register_node(&my_logger.node_data().name, serde_json::to_string(&my_logger), &mut node_vec);

        // ADD NEW NODE HERE

        node_vec
    }

    fn setup_constructor_map() -> HashMap<String, String> {

        let mut node_constructor_map: HashMap<String, String> = HashMap::new();
        let node_vec = NodeMap::get_node_constructor_vector();

        for entry in &node_vec {
            let node_name: String = entry.0.clone();
            let json_str: String = entry.1.clone();
            node_constructor_map.insert(node_name, json_str);
        }

        node_constructor_map
    }

    pub fn new() -> Self {
        NodeMap {
            // node_map: NodeMap::setup_constructor_map(),
            node_map: NodeMap::setup_constructor_map(),
        }
    }

    pub fn get_node(&self, node_name: String) -> Nodes {
        let json_str = self.node_map.get(&node_name);
        if json_str.is_none() {
            return Nodes::Invalid
        }

        let result = match node_name.as_str() {
            "PassthroughNode" => PassthroughNode::make_node(json_str.unwrap()),
            "AppendNode" => AppendNode::make_node(json_str.unwrap()),
            "LoggerNode" => LoggerNode::make_node(json_str.unwrap()),
            // ADD NEW NODE HERE
            _ => Nodes::Invalid,
        };

        result
    }
}

#[derive(Debug, Clone)]
pub struct NodeItem {
    pub a_node: Nodes,
    pub connections: Vec<NodeItem>,
}

impl NodeItem {

    pub fn new(a_node: Nodes) -> Self {
        NodeItem {
            a_node,
            connections: Vec::new(),
        }
    }

    pub fn get_number_of_connections(&self) -> usize {
        self.connections.len()
    }

    pub fn add_node_item_connection(&mut self, node_item: NodeItem) {
        self.connections.push(node_item);
    }
}

#[derive(Debug, Clone)]
pub struct NodeNetwork {
   pub top_nodes: Vec<NodeItem>
}

impl NodeNetwork {

    pub fn new() -> Self {
        NodeNetwork {
            top_nodes: Vec::new(),
        }
    }

    pub fn get_number_of_nodes(&self) -> usize {
        self.top_nodes.len()
    }

    pub fn add_node_to_network(&mut self, a_node_item: NodeItem) {
        self.top_nodes.push(a_node_item);
    }
}

#[derive(Debug, Clone)]
pub struct Configurator {
    node_map: NodeMap,
}

impl Configurator {

    pub fn new() -> Self {
        Configurator {
            node_map: NodeMap::new() ,
        }
    }

    pub fn get_nodes_for_node(&mut self, node_name: String) -> Nodes {
        self.node_map.get_node(node_name)
    }

    pub fn get_context_from_nodes(a_node: &mut Nodes) -> FBPNodeContext {

        let result: FBPNodeContext = match a_node {
            Nodes::PassthroughNode(ptn) => {
                let mut mut_pt_node:  PassthroughNode = ptn.as_ref().unwrap().clone();
                let mut_pt_nd = mut_pt_node.node_data_mut();
                mut_pt_nd.clone()
            },
            Nodes::AppendNode(ptn) => {
                let mut mut_an_node: AppendNode = ptn.as_ref().unwrap().clone();
                let mut_an_nd = mut_an_node.node_data_mut();
                mut_an_nd.clone()
            },
            Nodes::LoggerNode(ptn) => {
                let mut mut_ln_node: LoggerNode = ptn.as_ref().unwrap().clone();
                let mut_ln_nd = mut_ln_node.node_data_mut();
                mut_ln_nd.clone()
            }

            // ADD NEW NODE HERE
            Nodes::Invalid => FBPNodeContext::new("Invalid").clone(),
        };

        result
    }

    fn create_node(&mut self, a_config_node: &ConfigNode) -> Nodes {

       // Use the get_node method to convert the name of a node into a 'real' node
       let mut nodes_for_config_node = self.node_map.get_node(a_config_node.node_name.clone());

       // Ensure that the newly created node is valid.
       let context: FBPNodeContext  = Configurator::get_context_from_nodes(&mut nodes_for_config_node);

       // Do we have a 'real' node?
       if context.name == "Invalid".to_string() {
           println!("Failed to create {} node", a_config_node.node_name.clone());
          return Nodes::Invalid;
       }

       // If this node need to be configured then send the configuration message(s)
       for config_str in &a_config_node.configurations {

           let my_config_message = ConfigMessage {
               msg: ConfigMessageType::Field,
               data: Some(config_str.to_string()),
           };

           let msg = my_config_message.make_message(MessageType::Config);
           context.post_msg(msg);
       }

       nodes_for_config_node
    }

    fn create_connection(&mut self, config_network: &ConfigNodeNetwork,  parent_node_item: &mut NodeItem) {

        let mut parent_context = Configurator::get_context_from_nodes(&mut parent_node_item.a_node);

        for connection_config_node in &config_network.node_network {

            let connection_node = self.create_node(connection_config_node);

            if let Nodes::Invalid = connection_node {
                println!("Failed to create the {} node", connection_config_node.node_name);
                continue;
            }

            let mut connection_node_item = NodeItem::new(connection_node);



            // The dreaded Recursion!
            if connection_config_node.connections.node_network.len() > 0 {
                self.create_connection(&connection_config_node.connections, &mut connection_node_item);
            }


            let mut connection_node_context = Configurator::get_context_from_nodes(&mut connection_node_item.a_node);
            parent_context.add_receiver(&mut connection_node_context);
            parent_node_item.connections.push(connection_node_item);
        }
    }

    pub fn create_network_from_json(&mut self, json_str: String) -> NodeNetwork {

        // Create the NodeNetwork result
        let mut my_node_network = NodeNetwork::new();

        // From the given json_string, create the description of the network as a
        // Rust struct
        let node_network: ConfigNodeNetwork = serde_json::from_str(json_str.as_str())
            .expect("Failed to convert the JSON string into a ConfigNodeNetwork struct");

        // From the Rust struct that defines the network start the creation process
        // Loop through all of the nodes in the top level of the network, 99% of the
        // time there will be only one top level node.
        for a_config_node in node_network.node_network {

            let current_node = self.create_node(&a_config_node);

            if let Nodes::Invalid = current_node {
                println!("Failed to create the {} node", a_config_node.node_name);
            }

            let the_connection = &a_config_node.connections;

            let mut a_config_node_node_item = NodeItem::new(current_node);
            if the_connection.node_network.len() > 0 {
                self.create_connection(the_connection, &mut a_config_node_node_item);
            }

            my_node_network.add_node_to_network(a_config_node_node_item);
        }

        my_node_network
    }
}