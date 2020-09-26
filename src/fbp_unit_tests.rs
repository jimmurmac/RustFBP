
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

    use std::{thread, time};
    use crate::fbp_types::*;
    use crate::fbp_test_nodes::*;
    use crate::fbp_configurator::*;

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

        node.add_receiver(&mut other_node);
        assert_eq!(node.output_vec.lock().unwrap().len(), 1);

        node.remove_receiver(&mut other_node);
        assert_eq!(node.output_vec.lock().unwrap().len(), 0);
    }

    /* ----------------------------------------------------------------------
        Test creating and running a FBP network
        ---------------------------------------------------------------------- */
    #[test]
    fn test_run_node() {

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
        let pt_node = node_network.add_node("PassthroughNode".to_string()).unwrap();
        let an_node = pt_node.add_connection("AppendNode".to_string()).unwrap();
        let an_node_config_str = "{\"append_data\": \" World\"}".to_string();
        an_node.add_configuration(an_node_config_str);
        let lg_node = an_node.add_connection("LoggerNode".to_string()).unwrap();
        let lg_node_config_str = "{\"log_file_path\":\"Log_file.txt\"}".to_string();
        lg_node.add_configuration(lg_node_config_str);

        // Now that the FBP network has been described, turn it into a JSON string
        let node_network_json_str = serde_json::to_string(&node_network);
        let derived_network_str = node_network_json_str.unwrap();
        
        // The network_str is the JSON string that the configured FBP network 
        // should generate.  To ensure that, the derived_network_str is tested
        // for equality with this network_str.
        let network_str = r#"{"node_network":[{"node_name":"PassthroughNode","configurations":[],"connections":{"node_network":[{"node_name":"AppendNode","configurations":["{\"append_data\": \" World\"}"],"connections":{"node_network":[{"node_name":"LoggerNode","configurations":["{\"log_file_path\":\"Log_file.txt\"}"],"connections":{"node_network":[]}}]}}]}}]}"#;
        assert_eq!(derived_network_str, network_str);

        // Create an instance of the Configurator that will turn the JSON string
        // into the described FBP network
        let mut the_configurator = Configurator::new();
        let app_network = the_configurator.create_network_from_json(network_str.to_string());

        // Give the parameterization message a bit of time to complete.
        thread::sleep(time::Duration::from_secs(1));

        // Get the first node from the FBP network.  This will be the node
        // that will receive data messages.
        assert_eq!(app_network.get_number_of_nodes(), 1); 
        let top_node = &app_network.top_nodes[0];

        // In Rust, the return value of a function is part of the type 
        // signature of the function.  This means that there needs to
        // be a way of returning ANY node in the network of nodes.  This
        // is done using the Nodes enumeration.  In Rust enumerations can
        // be associated with a value.  The if let construct allows for
        // getting that value from a enumeration.  
        let mut pt_node: Option<&PassthroughNode> = None;
        if let Nodes::PassthroughNode(a_pass_node) = &top_node.a_node {
            let pt_node_ref = a_pass_node.as_ref().unwrap();
            pt_node = Some(pt_node_ref);
        }

        // Normally only the first node will need to be acquired as it 
        // will ensure that messages will flow through the network.
        // In this case, the last node which is the LoggerNode needs
        // to be acquired to that the get_log_string method can be
        // called as part of the unit test.  To get the last method
        // the network needs to be transversed.  First the append_node
        // is acquired from the top_node.  The ln_node is then
        // acquired from the append_node.
        let append_node = &top_node.connections[0];
        let ln_node = &append_node.connections[0];

        // Using the if let construct the actual logger_node is acquired.
        let mut logger_node: Option<&LoggerNode>  = None;
        if let Nodes::LoggerNode(a_logger_node) = &ln_node.a_node {
            let logger_node_ref = a_logger_node.as_ref().unwrap();
            logger_node = Some(logger_node_ref);
        } 

        // Create a message to send through the FBP network and send
        // it to the top most node, in the case the pt_node.
        let my_msg = IIDMessage::new(MessageType::Data, Some("Hello".to_string()));
        if pt_node.is_some() {
            pt_node.unwrap().node_data().post_msg(my_msg);
        }

        // Get the network a bit of time to complete all of the processing
        thread::sleep(time::Duration::from_secs(1));

        // Using the logger_node acquired previously, get the contents
        // of the log file and test that against the expected value.
        let log_string_result = logger_node.unwrap().get_log_string();
        if log_string_result.is_ok() {
            let log_string = log_string_result.unwrap();
            assert_eq!(log_string, "Hello World");
        } else {
            println!("Failed to get the log string");
        }

        // Create a process message to stop all of the nodes in the
        // FBP neetwork
        let stop_msg = ProcessMessage::new(ProcessMessageType::Stop, true);
        let stop_msg_string = serde_json::to_string(&stop_msg).unwrap();

        // Create an IIDMessage with the JSON string.
        let the_stop_msg = IIDMessage::new(MessageType::Process,
                                           Some(stop_msg_string));

        // Send the message to the first node in the netowrk
        pt_node.unwrap().node_data().post_msg(the_stop_msg);

        // Give the message a bit of time to complete processing.
        thread::sleep(time::Duration::from_secs(1));

        // Unit test complete!
        println!("That's all folks.");
    }
}