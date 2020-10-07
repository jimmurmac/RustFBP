use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fbp_message_types::*;

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
                   node_network: HashMap::new(),
               },
           }
       }
   
       pub fn add_configuration(&mut self, config_str:String) {
           let configs = &mut self.configurations;
           configs.push(config_str);
           // self.configurations.push(config_str);
       }
   
       pub fn add_connection(&mut self, node_name: String, key: Option<String>) ->  Option<&mut ConfigNode> {
            let mut hash_key = "Any".to_string();
            
            if key.is_some() {
                hash_key = key.unwrap();
            }
        
            let config_node = self.connections.add_node(node_name, Some(hash_key));
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
       pub node_network: HashMap<String, Vec<ConfigNode>>,
   }
   
   #[allow(dead_code)]
   impl ConfigNodeNetwork {
   
       pub fn new() -> Self {
           ConfigNodeNetwork {
               node_network: HashMap::new(),
           }
       }
   
       pub fn find_node(&mut self, node_name: String, key: Option<String>) -> Option<&mut ConfigNode> {
   
            let mut hash_key = "Any".to_string();

            if key.is_some() {
                hash_key = key.unwrap();
            }

            let mut result: Option<&mut ConfigNode> = None;
            let op_my_vec = self.node_network.get_mut(&hash_key);

            if op_my_vec.is_some() {
                let my_vec = op_my_vec.unwrap();
                for a_config_node in my_vec {

                    if a_config_node.node_name == node_name {
                        result = Some(a_config_node);
                        break;
                    }
                }
            }

            
   
            result
       }
   
       pub fn add_node(&mut self, node_name: String, key: Option<String>) -> Option<&mut ConfigNode> {

            let mut hash_key = "Any".to_string();

            if key.is_some() {
                hash_key = key.clone().unwrap();
            }

            // First create the node.
            let nn = node_name.clone();
            let node_config = ConfigNode {
                node_name,
                configurations: Vec::new(),
                connections: ConfigNodeNetwork {
                    node_network: HashMap::new(),
                },
            };

            if  self.node_network.get_mut(&hash_key).is_some() {
                let a_node_vec = self.node_network.get_mut(&hash_key).unwrap();
                a_node_vec.push(node_config);
            } else {
                let mut config_vec: Vec<ConfigNode> = Vec::new();
                config_vec.push(node_config);
                self.node_network.insert(hash_key, config_vec);
            }

            self.find_node(nn, key.clone())
       }
   }
   
   impl MessageSerializer for ConfigNodeNetwork{}