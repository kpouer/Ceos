use crate::textarea::buffer::Buffer;

pub(crate) enum Event {
    BufferLoaded(Buffer),
}
