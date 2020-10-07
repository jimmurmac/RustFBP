use futures::*;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};
use serde::{Deserialize, Serialize};
use std::ops::{Deref};

/* --------------------------------------------------------------------------
    struct AsyncState

    Defines the a type that can use await to wait on a state change

    Fields:
        value_is_ready:
                    This is the AtomicBool that is used to 'signal' that
                    a particular item is 'ready'.  If await is used on 
                    this type then it will wait until this value is set
                    to true.
   -------------------------------------------------------------------------- */             
   #[derive(Debug, Clone, Serialize, Deserialize)] 
   pub struct AsyncState {
   
       #[serde(skip)]
       value_is_ready: Arc<AtomicBool>,
   }
   
   /* --------------------------------------------------------------------------
       Implement the Future trait for the AsyncState type.  This will allow
       a user to do the following:
   
       let my_async_state = AsyncState::new();
   
       my_async_state.await;
   
       NOTE:  The call to await WILL block the current thread until the item 
       for which the AsyncState is waiting for becomes true.  This means that 
       another thread MUST call my_async_state.set_is_ready(true) to have the
       waiting thread continue.  
   
       NOTE: The AtomicBool which is the state that determines if the AsyncState
       has completed is in an Arc which means that a clone of the original 
       AsyncState will still point to the original AtomicBool so cloning for a 
       thread will be OK.
      -------------------------------------------------------------------------- */             
   impl Future for AsyncState {
       type  Output = bool;
   
       // Wait until the state is ready.
       fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
           if self.is_ready() {
               Poll::Ready( true )
           } else {
               cx.waker().wake_by_ref();
               Poll::Pending
           }
       }
   }
   
   impl AsyncState {
       pub fn new() -> Self {
           AsyncState {
               value_is_ready: Arc::new(AtomicBool::new(false)),
           }
       }
   
       pub fn is_ready(&self) -> bool {
           self.value_is_ready.deref().load(Ordering::Relaxed)
       }
   
       pub fn set_is_ready(&self, flag: bool) {
           self.value_is_ready.store(flag, Ordering::Relaxed)
       }
   }