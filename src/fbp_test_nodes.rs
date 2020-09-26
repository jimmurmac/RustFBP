/* ==========================================================================
    File:           fbp_types.rs

    Description:    This file contains the 'test' nodes used by the 
                    fbp_unit_tests.rs file.  While these nodes   
                   
    History:        Jim Murphy 08/07/2020   Initial Code.
                    Jim Murphjy 09/13/2020  Got all of the unit tests passing.

    Copyright ©  2020 Pesa Switching Systems Inc. All rights reserved.
   ========================================================================== */

    // use std::{thread, time};
    use std::fs::File;
    use std::io;
    use std::io::Read;
    use std::io::prelude::*;
    use std::io::{Error, ErrorKind};
    use std::fs::OpenOptions;
    use serde::{Deserialize, Serialize};
    use crate::fbp_types::*;
    use serde_json::json;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::ops::{DerefMut, Deref};
    use crate::fbp_configurator::*;

     /* ----------------------------------------------------------------------
        To allow for testing node interactions, this unit test creates three
            nodes:

            PassthroughNode:    This node is a "null" node.  It just passes it's
                                input to it's output

            AppendNode:         This node will append a string to the message's
                                it receives and then sends them onto the next
                                node.

            LoggerNode:         This node will write out the payload of the
                                 data messages that are sent to it to a file.
    ---------------------------------------------------------------------- */

    #[derive(Debug, Clone, Serialize, Deserialize)] //PartialEq)]
    pub struct PassthroughNode {
        data: Box<FBPNodeContext>,
    }

    impl PassthroughNode {

        pub fn new() -> Self {
           let result =  PassthroughNode {
                data: Box::new(FBPNodeContext::new("PassthroughNode")),
            };

            result.clone().start();
            result
        }
    }

    impl NodeConstructor for PassthroughNode {

        fn make_node(json_str: &String) -> Nodes {
            let mut fn_result = Nodes::Invalid;
            let node_result: Result<Option<PassthroughNode>, serde_json::Error> = serde_json::from_str(json_str);
            if node_result.is_ok() {
                let node_option = node_result.unwrap().clone();
                if node_option.is_some() {
                    fn_result = Nodes::PassthroughNode(node_option.clone());
                    node_option.unwrap().clone().start();
                }
            }
            fn_result
        }
    }

    impl FBPNode for PassthroughNode {
        fn node_data(&self) -> &FBPNodeContext { &self.data }

        fn node_data_mut(&mut self) -> &mut FBPNodeContext { &mut self.data }

        fn node_is_configured(&self) -> bool {
            true
        }

        fn process_message(&self, msg: IIDMessage) -> Result<IIDMessage, NodeError> {
            // Simply pass on the message that was sent to this node.
            Ok(msg.clone())
        }
    }


    #[derive(Debug, Clone, Serialize, Deserialize)] // PartialEq)]
    pub struct AppendNode {
        data: Box<FBPNodeContext>,

        #[serde(skip)]
        append_data: Arc<Mutex<Option<String>>>,
    }

    impl AppendNode {
        pub fn new() -> Self {
            let result = AppendNode {
                data: Box::new(FBPNodeContext::new("AppendNode")),
                append_data: Arc::new(Mutex::new(None)), // Box::new(None),
            };

            result.clone().start();
            result
        }

        pub fn set_append_data(&mut self, data: String) {
            *self.append_data.lock().unwrap().deref_mut() = Some(data);
        }
    }

    impl NodeConstructor for AppendNode {

        fn make_node(json_str: &String) -> Nodes {
            let mut fn_result = Nodes::Invalid;
            let node_result: Result<Option<AppendNode>, serde_json::Error> = serde_json::from_str(json_str);
            if node_result.is_ok() {
                let node_option = node_result.unwrap().clone();
                if node_option.is_some() {
                    fn_result = Nodes::AppendNode(node_option.clone());
                    node_option.unwrap().clone().start();
                }
            }
            fn_result
        }
    }

    impl FBPNode for AppendNode   {
        fn node_data(&self) -> &FBPNodeContext {&self.data}

        fn node_data_mut(&mut self) -> &mut FBPNodeContext {&mut self.data}

        fn node_is_configured(&self) -> bool {
            self.append_data.lock().unwrap().is_some()
        }

        fn process_config(&mut self, msg:IIDMessage) ->  std::result::Result<IIDMessage, NodeError> {
            if msg.msg_type == MessageType::Config {
                if msg.payload.is_some() {
                    let payload = msg.payload.unwrap();
                    let config_message: ConfigMessage = serde_json::from_str(&payload)
                        .expect("Failed to deserialize the config message");
                    if config_message.msg == ConfigMessageType::Field {
                        if config_message.data.is_some() {
                            let config_str  = json!(config_message.data.unwrap());
                            let key_str = "append_data";
                            if config_str.to_string().contains(key_str) {
                                let json_str = config_str.as_str().unwrap();
                                let append_str = get_value_from_json_map_key(json_str, key_str);
                                self.set_append_data(append_str);
                            }
                        } // if config_message.data.is_some()
                    } // if config_message.msg == ConfigMessageType::Field
                } //  if msg.payload.is_some()
            } // if msg.msg_type == MessageType::Config

            Ok(IIDMessage::new(MessageType::Invalid, None))
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {

            let string_ref = self.append_data.lock().unwrap().deref().as_ref().unwrap().clone();

            if msg.payload.is_some() {
                let mut payload = msg.payload.unwrap().clone();
                if self.append_data.lock().unwrap().is_some() {
                    payload.push_str(string_ref.as_str());
                }

                let new_msg = IIDMessage::new(MessageType::Data, Some(payload));
                return Ok(new_msg);
            } else {
                if self.append_data.lock().unwrap().is_some() {
                    let new_msg = IIDMessage::new(MessageType::Data, Some(string_ref));
                    return Ok(new_msg);
                }
            }

            Ok(msg.clone())
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoggerNode{
        data: Box<FBPNodeContext>,

        #[serde(skip)]
        log_file_path: Arc<Mutex<Option<String>>>,
    }


    #[allow(dead_code)]
    impl LoggerNode {

        pub fn new() -> Self {

            let result = LoggerNode {
                data:  Box::new(FBPNodeContext::new("LoggerNode")),
                log_file_path: Arc::new(Mutex::new(None)),
            };

            result.clone().start();
            result
        }


        pub fn set_log_file_path(&mut self, log_file_path: String) {

            *self.log_file_path.lock().unwrap().deref_mut() = Some(log_file_path);

            // Ensure the File
            let string_ref = self.log_file_path.lock().unwrap().as_ref().unwrap().clone();
            let file_path = Path::new(string_ref.as_str());
            let file = File::create(file_path).expect ("Unable to create file");
            drop(file)
        }

        pub fn get_log_string(&self) -> io::Result<String> {

            if self.log_file_path.lock().unwrap().is_none() {
                return Err(Error::new(ErrorKind::Other, "Cannot get log string until the node is setup"))
            }

            let mut contents = String::new();
            let string_ref = self.log_file_path.lock().unwrap().as_ref().unwrap().clone();

            let file_path = Path::new(string_ref.as_str());
            let mut file = OpenOptions::new().read(true)
                .open(file_path)
                .expect("Failed to open file {} for reading");

            file.read_to_string(&mut contents)
                .expect("Failed to write contents to string");

            drop(file);
            Ok(contents)
        }

        pub fn log_string_to_file(&self, data: &String) -> io::Result<()> {

            if self.log_file_path.lock().unwrap().is_none() {

                return Err(Error::new(ErrorKind::Other, "Cannot get log to file until the node is setup"))
            }

            let string_ref = self.log_file_path.lock().unwrap().as_ref().unwrap().clone();
            let file_path = Path::new(string_ref.as_str());

            let mut file = OpenOptions::new().append(true)
                .open(file_path)
                .expect("Failed to open file for append");

            let _write_result = file.write(data.as_bytes());
            drop(file);
            Ok(())
        }
    }

    impl NodeConstructor for LoggerNode {

        fn make_node(json_str: &String) -> Nodes {
            let mut fn_result = Nodes::Invalid;
            let node_result: Result<Option<LoggerNode>, serde_json::Error> = serde_json::from_str(json_str);
            if node_result.is_ok() {
                let node_option = node_result.unwrap().clone();
                if node_option.is_some() {
                    fn_result = Nodes::LoggerNode(node_option.clone());
                    node_option.unwrap().clone().start();
                }
            }
            fn_result
        }
    }

    impl FBPNode for LoggerNode   {
        fn node_data(&self) -> &FBPNodeContext {&self.data}

        fn node_data_mut(&mut self) -> &mut FBPNodeContext {&mut self.data}

        fn node_is_configured(&self) -> bool {
            self.log_file_path.lock().unwrap().is_some()
        }

        fn process_config(&mut self, msg:IIDMessage) ->  std::result::Result<IIDMessage, NodeError> {
            if msg.msg_type == MessageType::Config {
                if msg.payload.is_some() {
                    let payload = msg.payload.unwrap();
                    let config_message: ConfigMessage = serde_json::from_str(&payload)
                        .expect("Failed to deserialize the config message");
                    if config_message.msg == ConfigMessageType::Field {
                        if config_message.data.is_some() {
                            let config_str  = json!(config_message.data.unwrap());
                            let key_str = "log_file_path";
                            if config_str.to_string().contains(key_str) {
                                let json_str = config_str.as_str().unwrap();
                                let file_path_string = get_value_from_json_map_key(json_str, key_str);
                                self.set_log_file_path(file_path_string);
                            }
                        }
                    }
                }
            }

            Ok(IIDMessage::new(MessageType::Invalid, None))
        }

        fn process_message(&self, msg: IIDMessage) ->  Result<IIDMessage, NodeError> {
            if msg.payload.is_some() {
                let payload = msg.clone().payload.unwrap();
                if self.log_string_to_file(&payload).is_err() {
                    return Err(NodeError::new("Failed to write message to log file"));
                }
            }

            Ok(msg.clone())
        }
    }

