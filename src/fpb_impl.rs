// ----------------------------------------------------------------------------
// trait FPBChannel
//
// Define a trait that defines the basic behavior for a FPBNode
// ----------------------------------------------------------------------------

trait FPBChannel {
    type MessageItem;

    fn post_work(&self, job: IIDMessage<Self::MessageItem>);

    fn process_work(&self);

    fn start(&self);

    fn stop(&self);
}