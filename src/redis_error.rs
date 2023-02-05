use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyRedisError{
    #[error("Inavlid Redis Syntax ({message})")]
    InvalidRedisSynatx{message: String},
    
    #[error("Index Error")]
    IndexError{message: String},

    #[error("Unsupported Structure: Cannot use array in array. Use json or similiar data formats instead")]
    UnsupportedStructure{},
}