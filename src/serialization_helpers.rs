/* ==========================================================================
    File:           serialization_helpers.rs

    Description:    traits for serializing and deserializing Rust structs

    History:        Jim Murphy 03/23/2022  

    Copyright ©  2022 Pesa Switching Systems Inc. All rights reserved.
   ========================================================================== */

   #[allow(unused_imports)]
   #[cfg(not(debug_assertions))]
   use log::{error, warn, info, debug, trace};
   
   #[allow(unused_imports)]
   #[cfg(debug_assertions)]
   use std::{println as error, println as warn, println as info, println as debug, println as trace};
   
   pub trait StructDeserializer {
       fn make_struct_from_string<'a, T>(json_str: &'a str) -> Result<T, serde_json::Error>
           where T: std::marker::Sized + serde::Deserialize<'a>,
       {
           serde_json::from_str(json_str)
       }
   }
   
   
   pub trait StructSerializer {
       fn make_string_from_struct(&self) -> Result<String, serde_json::Error> 
       where 
           Self: std::marker::Sized + serde::Serialize
       {
           serde_json::to_string(&self)
       }
   }
   
  