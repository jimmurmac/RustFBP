
/* ==========================================================================
    Unit Test


   ========================================================================== */

/* --------------------------------------------------------------------------
    mod test

    Description:    Provide testing for the FBP_types file.

    Tests:
        test_iidmessage:

            Description:    Do testing for the struct IIDMessage. Creates a
                            IIDMessage and ensures that fields are correct.

        test_fbpnode_context:

            Description:    Do testing for the struct FBPNodeContext. Creates
                            a FBPNodeContext and ensures that the fields are
                            correct.  Another FBPNodeContext is then created
                            and added as a receiver for the first node.  The
                            number of items in the output_vec are checked to
                            ensure that the receiver was added.  The receiver
                            is then removed and the output_vec is checked to
                            ensure that it is now empty.

        test_run_node:

            Description:    This is the full Flow Based Programming test.  It
                            creates three nodes.
                                1)  A PassthroughNode which simply passes it's
                                    to it's output.

                                2)  An AppendNode which will take an incoming
                                    message and 'append' a string to the
                                    message. NOTE: This is a crude
                                    implementation only for testing.  A real
                                    AppendNode would need to be able to
                                    determine the best way to append.

                                3)  A LoggerNode which will take an incoming
                                    message and append it to a file for
                                    logging purposes.

                            The AppendNode and the LoggerNode need to be
                            configured BEFORE they can process messages.
                            Part of this test is to create a ConfigMessage
                            to configure both the AppendNode and the LoggerNode.
                            Once configured, the nodes are put into a network
                            of just three nodes.  A data message is sent to
                            the first node which in turn will be processed by
                            the other two nodes.  In the end the contents of
                            the log file created by the Logger Node is tested
                            to ensure that the original message was appended
                            to by the AppendNode and that the output is as
                            expected.
   -------------------------------------------------------------------------- */

#[cfg(test)]
mod test {    
    use std::path::Path;
    use std::fs::OpenOptions;
    use std::io::Read;

    use crate::fbp_configurator::*;
    use crate::fbp_message_types::*;
    use crate::fbp_iidmessage::*;
    use crate::fbp_nodecontext::*;
    use crate::fbp_confignode::*;

    /* ----------------------------------------------------------------------
       Read the contents of a file into a string
      ---------------------------------------------------------------------- */
    pub fn get_logger_file_string(path_str: &str) -> String {
        let file_path = Path::new( path_str);
        let mut file = OpenOptions::new().read(true)
            .open(file_path)
            .expect("Failed to open file {} for reading");

        let mut result: String   = "".to_string();
        file.read_to_string(&mut result).expect("Failed to write contents to string");
        result
    }

    /* ----------------------------------------------------------------------
       Test the IIDMessage struct
      ---------------------------------------------------------------------- */
    #[test]
    fn test_iidmessage() {
        let msg = IIDMessage::new(MessageType::Data, Some("foo".to_string()));

        assert_eq!(msg.msg_type, MessageType::Data);
        assert_eq!(msg.payload.is_none(), false);
    }

    /* ----------------------------------------------------------------------
      Test the FBPNodeContext struct
     ---------------------------------------------------------------------- */
    #[test]
    fn test_fbpnode_context() {
        let mut node = FBPNodeContext::new(&"TestNode");

        assert_eq!(node.name, "TestNode".to_string());
        assert_eq!(node.node_is_running(), false);

        let mut other_node = FBPNodeContext::new(&"OtherNode");

        node.add_receiver(&mut other_node, None);

        let num_items = node.get_num_items_for_receiver_vec(None);
        assert_eq!(num_items, 1);

        node.remove_receiver(&mut other_node, None);

        let num_items = node.get_num_items_for_receiver_vec(None);
        assert_eq!(num_items, 0);
    }

    /* ----------------------------------------------------------------------
        Test creating and running a FBP network
        ---------------------------------------------------------------------- */
    #[actix_rt::test]
    async fn test_run_node() {

    /* ----------------------------------------------------------------------
        The Configurator takes a JSON string and turns it into a FBP network
        of nodes.  Beyond creating the nodes, the  Configurator will also 
        send the neccessary configuration (parameterization) messages to the
        newly created nodes. It will also setup the network connections 
        between these nodes so that the output of the
        Configurator::create_network_from_json function is a fully realized
        FBP network.  

        The JSON string needed to create a FBP network can be created using
        a ConfigNodeNetwork struct.  With this struct, a FBP network can
        be described and if necessary, connections and configurations for 
        the nodes in the network can be described.  
       ---------------------------------------------------------------------- */

        

        /* ------------------------------------------------------------------
            Use that instance to describe the FBP network that needs to be created.
            In this case a three node network as follows:
        
            PassthroughNode -> AppendNode -> LoggerNode
        
            The connections from one node to another node is done using the
            ConfigNode::add_connection method.
        
            The Append node needs to be parameterized by setting the append_data
            member.  This is done by calling ConfigNode::add_configuration
        
            The Logger node needs to be parameterized by setting the log_file_path
            member. 
        
            Once the FBP network has been described and all of the necessary
            parameterization has been described, the Rust ConfigNodeNetwork
            is serialized into a JSON string.  This JSON string can be used
            by the Configurator to create the described FBP network.

            NOTE: While the ability to create the FBP network JSON 
            programmatically is available and is shown in thei Unit test, that
            most likely will NOT be the only to create FBP network definitions.
            Eventually, a 'visual editor' will need to be created that will
            allow for creating a FBP network and this 'visual editor' will 
            output a JSON string that thhe Configurator will use.  This is
            shown in this Unit test by the network_str string. It is just
            a JSON string and it is used to create the FBP network.
        ---------------------------------------------------------------------- */
    
        // Create a ConfigNodeNetwork instance to allow for describing a FBP
        // network
        let mut node_network = ConfigNodeNetwork::new();

        // Define the network using the ConfigNodeNetwork instance.  This
        // includes saying what nodes are in the network, how they are
        // connected and any necessary parameterization.
        let pt_node = node_network.add_node("PassthroughNode".to_string(), None).unwrap();
        let an_node = pt_node.add_connection("AppendNode".to_string(), None).unwrap();
        let an_node_config_str = "{\"append_data\": \" World\"}".to_string();
        an_node.add_configuration(an_node_config_str);
        let lg_node = an_node.add_connection("LoggerNode".to_string(), None).unwrap();
        let lg_node_config_str = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
        lg_node.add_configuration(lg_node_config_str);

        // Now that the FBP network has been described, turn it into a JSON string
        let node_network_json_str = serde_json::to_string(&node_network);
        let derived_network_str = node_network_json_str.unwrap();
        
        // The network_str is the JSON string that the configured  network
        // should generate.  To ensure that, the derived_network_str is tested
        // for equality with this network_str.
        let network_str = r#"{"node_network":{"Any":[{"node_name":"PassthroughNode","configurations":[],"connections":{"node_network":{"Any":[{"node_name":"AppendNode","configurations":["{\"append_data\": \" World\"}"],"connections":{"node_network":{"Any":[{"node_name":"LoggerNode","configurations":["{\"log_file_path\":\"Log_file.txt\"}"],"connections":{"node_network":{}}}]}}}]}}}]}}"#;

        assert_eq!(derived_network_str, network_str);

        // Create an instance of the Configurator that will turn the JSON string
        // into the described FBP network
        let mut the_configurator = Configurator::new();
        let mut app_network = the_configurator.create_network_from_json(network_str.to_string());

        // Get the first node from the FBP network.  This will be the node
        // that will receive data messages.
        assert_eq!(app_network.get_number_of_nodes(), 1); 
        let top_node = &mut app_network.top_nodes[0];

        // Wait for all of the nodes to start
        the_configurator.wait_for_nodes_to_all_be_running().await;

        // Wait for all of the nodes to be configured
        the_configurator.wait_for_nodes_to_be_configured().await;

        // Now that the node network has been created, get the FBPContext
        // of the first node in the network.
        let node_context = Configurator::get_context_from_nodes(&mut top_node.a_node);

        // Create a data message and send it through the network
        let data_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        node_context.post_msg(data_msg);

        // Create a stop process message and send it through the network.
        let stopper = ProcessMessage::new(ProcessMessageType::Stop, true).make_message(MessageType::Process);
        node_context.post_msg(stopper);

        // Wait for all of the nodes to complete.
        the_configurator.wait_for_nodes_to_all_complete().await;

        // While there is a method on the Logger Node to read out the LoggerNode
        // file, that would require getting the LoggerNode out of the network.
        // Reading the file directly saves having to do that.
        let  file_contents = get_logger_file_string("Log_file.txt");
        assert_eq!(file_contents, "Hello World");

        // Unit test complete!
        println!("That's all folks.");
    }
}