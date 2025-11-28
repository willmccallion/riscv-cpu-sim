mod decode;
mod execute;
mod fetch;
mod mem;
mod write_back;

pub use decode::decode_stage;
pub use execute::execute_stage;
pub use fetch::fetch_stage;
pub use mem::mem_stage;
pub use write_back::wb_stage;
