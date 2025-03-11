
use crate::encoder::EncoderOpts;

pub trait Stage<T, V> {
    fn encode(data: T, encoder_opts: EncoderOpts) -> Result<V, String>;
    fn decode(data: V) -> Result<T, String>;
}
