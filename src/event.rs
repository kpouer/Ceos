use crate::ceos::textarea::buffer::Buffer;

pub(crate) enum Event {
    BufferLoaded(Buffer),
}
