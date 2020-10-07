use std::fmt;
use std::error::Error;

/* --------------------------------------------------------------------------
    struct NodeError

    Defines an error type for nodes.  This is still a Work in Progress.  
    What needs to be done is to provide an enumeration of various node errors.

    Fields:
        details:    The details field will contain a human readable version 
                    of the error.
   -------------------------------------------------------------------------- */

   #[derive(Debug)]
   pub struct NodeError {
       details: String
   }
   
   /* --------------------------------------------------------------------------
       struct NodeError behavior
   
       Methods:
           new:
               Parameters: 
                   msg:        A human readable error message.
   
               Description:    This will create a new NodeError 
                               (Constructor)
      -------------------------------------------------------------------------- */
   
   #[allow(dead_code)]
   impl NodeError {
       pub fn new(msg: &str) -> NodeError {
           NodeError {details: msg.to_string()}
       }
   }
   
   /* --------------------------------------------------------------------------
       Implement the fmt::Display trait for the NodeError struct
      -------------------------------------------------------------------------- */
   
   impl fmt::Display for NodeError {
       fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
           write!(f, "{}", self.details)
       }
   }
   
   /* --------------------------------------------------------------------------
       Implement the Error trait for the NodeError struct
      -------------------------------------------------------------------------- */
   
   impl Error for NodeError {
       fn description(&self) -> &str {
           &self.details
       }
   }