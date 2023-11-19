pub use input_buffer::UserInput;

/// Handles dealing with inputs (keyboard presses, mouse clicks) sent from a player (client) to server
pub mod input_buffer;
mod plugin;

//
// - ClientInputs:
// - inputs will be sent via a special message
// - in each packet, we will send the inputs for the last 10-15 frames. Can use the ring buffer?
// - First: send all inputs for last 15 frames. Along with the tick at which the input was sent
// - Maybe opt: Only send the inputs that have changed and the tick at which they change?
// - in the client: we don't send a packet every tick. So what we do is:
// - during fixedupdate, we store the input for the given tick. Store that input in a ringbuffer containing the input history. (for at least the rollback period)
// - at the end of the frame, we collect the last 15 ticks of inputs and put them in a packet.
// - we send that packet via tick-buffered sender, associated with the last client tick
// - IS THIS CORRECT APPROACH? IT WOULD MEAN THAT WE WOULD READ THAT PACKET ONLY ON THE CURRENT TICK IN THE SERVER, BUT ACTUALLY WE WANT TO READ IT IMMEDIATELY
// (BECAUSE IT CONTAINS LAST 15 TICKS OF INPUTS, SO CAN HELP FILL GAPS IN INPUTS!)
// - IT WOULD SEEM THAT WE CAN JUST SEND THE PACKET AS SEQUENCED-UNRELIABLE. (WE DONT NEED TO KNOW THE PACKET TICK BECAUSE IT CONTAINS TICKS)
// ON THE SERVER WE READ IMMEDIATELY AND WE UPDATE OUR RINGBUFFER OF INPUTS THAT WE CAN FETCH FROM!
// - during rollback, we can read from the input history
// - the input history is associated with a connection.
// - in the server, we receive the inputs, open the packet, and update the entire ringbuffer of inputs?
// - server is at tick 9. for example we didn't receive the input for tick 10,11; but we receive the packet for tick 12, which contains all the inputs for ticks 10,11,12.