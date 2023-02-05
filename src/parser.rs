use crate::redis_error::MyRedisError;
use anyhow::{Context,Result,Ok};

#[derive(Debug)]
pub enum RedisType{
    Error(RedisError),
    SimpleString(RedisSimpleString),
    BulkString(RedisBulkString),
    Integer(RedisInteger),
    Array(RedisArray),
}

impl RedisType {
    pub fn parse(redis_string: String) -> Result<Option<RedisType>>{
        let first:usize = 0; 
        let cmd: RedisType;
        let type_char: String = match redis_string.chars().nth(first){
            None => return Ok(None),
            Some(c) => c.to_string()
        };
        if type_char == String::from("-"){
            let (content, _) = RedisError::parse(redis_string)?;
            return Ok(Some(RedisType::Error(content)))

        }else if type_char == String::from("+"){
            let (content, _) = RedisSimpleString::parse(redis_string)?;
            cmd = RedisType::SimpleString(content);

        }else if type_char == String::from(":"){
            let (content, _) = RedisInteger::parse(redis_string)?;
            cmd = RedisType::Integer(content);

        }else if type_char == "$"{
            let (content, _) = RedisBulkString::parse(redis_string)?;
            cmd = RedisType::BulkString(content);

        }else if type_char == "*".to_string(){
            let (content, _) = RedisArray::parse(redis_string)?;
            cmd = RedisType::Array(content);

        }else{
            return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("unknown Redis Type char: {}",type_char)})) 
        }
        Ok(Some(cmd))
    }

    pub fn get_value(&self) -> Option<Vec<String>>{
        if let RedisType::Error(er) = self{
            return Some(vec![er.get_value()?])

        }else if let RedisType::Array(ar) = self{
            return Some(ar.get_value()?)

        }else if let RedisType::Integer(int) = self{
            return Some(vec![int.get_value()?.to_string()]);

        }else if let RedisType::SimpleString(sstr) = self{
            return Some(vec![sstr.get_value()?])

        }else if let RedisType::BulkString(bstr) = self{
            return Some(vec![bstr.get_value()?])

        }else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RedisArray{
    array: Option<Vec<RedisType>>
}

#[derive(Debug)]
pub struct RedisError {
    error_msg: Option<String>,
}

#[derive(Debug)]
pub struct RedisInteger{
    integer: Option<i64>
}

#[derive(Debug)]
pub struct RedisSimpleString{
    content: Option<String>
}

#[derive(Debug)]
pub struct RedisBulkString{
    content: Option<String>
}


impl RedisArray {
    fn parse(mut redis_string: String) -> Result<(Self,String)>{
        let first:usize = 0; 
        let type_char: String = redis_string.chars().nth(first).unwrap().to_string();
        let mut string_size = String::new();
        for (i, c) in redis_string.chars().enumerate(){
            if c.is_ascii_control() && match redis_string.chars().nth(i+1){
                Some(c) => c.is_ascii_control(),
                None => return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("missing end of redis string (index: {} out of range for: {})",i.to_string(), redis_string)}))
            }{
                redis_string = redis_string[i+1..].to_string();
                break;
            }else{
                string_size.push(c);
            }
        }

        let size: i32 = string_size.parse().context(format!("Cant parse string_size: {} to size", string_size.to_string()))?;
        let mut ar: Vec<RedisType> = vec![];
        for _ in [..size]{
            if type_char == "-".to_string(){
                let (content, new_r) = RedisError::parse(redis_string)?;
                redis_string = new_r;
                ar.push(RedisType::Error(content));
    
            }else if type_char == "+".to_string(){
                let (content, new_r) = RedisSimpleString::parse(redis_string)?;
                redis_string = new_r;
                ar.push(RedisType::SimpleString(content));
    
            }else if type_char == ":".to_string(){
                let (content, new_r) = RedisInteger::parse(redis_string)?;
                redis_string = new_r;
                ar.push(RedisType::Integer(content));
    
            }else if type_char == "$".to_string(){
                let (content, new_r) = RedisBulkString::parse(redis_string)?;
                redis_string = new_r;
                ar.push(RedisType::BulkString(content));

            }else if type_char == "*".to_string(){
                return Err(anyhow::Error::new(MyRedisError::UnsupportedStructure {})) 
            }else{
                return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("unknown Redis Type char: {}",type_char)}))                
            }
        }
        //return Ok((RedisArray{array: None},redis_string))
        return Ok((RedisArray{array: Some(ar)},redis_string))


    }

    fn get_value(&self) -> Option<Vec<String>>{
        let vals = match &self.array{
            None => return None,
            Some(array) => array
        };
        let mut re: Vec<String> = vec![];
        for val in vals{
            match val.get_value(){
                Some(mut v) => re.append(&mut v),
                None => re.push("!NULL!".to_string())
            }
        }
        Some(re)
    }
}

impl RedisInteger{
    fn parse(mut redis_string: String) -> Result<(Self,String)>{
        redis_string = redis_string.trim_start_matches(":").to_string();
        let mut integer_string = String::new();
        for (i, c) in redis_string.chars().enumerate(){
            if c.is_ascii_control() && match redis_string.chars().nth(i+1){
                Some(c) => c.is_ascii_control(),
                None => return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("missing end of redis string (index: {} out of range for: {})",i.to_string(), redis_string)}))
            }{
                redis_string = redis_string[i+1..].to_string();
                break;
            }else{
                integer_string.push(c);
            }
        }
        Ok((Self{integer:Some(integer_string.parse::<i64>()?)},redis_string))
    }

    fn get_value(&self) -> Option<i64>{
        self.integer
    }
}

impl RedisSimpleString{
    fn parse(mut redis_string: String) -> Result<(Self,String)>{
        redis_string = redis_string.trim_start_matches("+").to_string();
        let mut cont = String::new();
        for (i, c) in redis_string.chars().enumerate(){
            if c.is_ascii_control() && match redis_string.chars().nth(i+1){
                Some(c) => c.is_ascii_control(),
                None => return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("missing end of redis string (index: {} out of range for: {})",i.to_string(), redis_string)}))
            }{
                redis_string = redis_string[i+1..].to_string();
                break;
            }else{
                cont.push(c);
            }
        }
        Ok((Self{content:Some(cont)},redis_string))
    }

    fn get_value(&self) -> Option<String>{
        match &self.content{
            Some(cont) => Some(cont.to_string()),
            None => None
       }
    }
}

impl RedisBulkString{
    fn parse(mut redis_string: String) -> Result<(Self,String)>{
        redis_string = redis_string.trim_start_matches("$").to_string();
        let mut string_size = String::new();
        for (i, c) in redis_string.chars().enumerate(){
            if c.is_ascii_control() && match redis_string.chars().nth(i+1){
                Some(c) => c.is_ascii_control(),
                None => return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("missing end of redis string (index: {} out of range for: {})",i.to_string(), redis_string)}))
            }{
                redis_string = redis_string[i+2..].to_string();
                break;
            }else{
                string_size.push(c);
            }
        }
        let size:usize = string_size.parse()?;
        Ok((Self{content: Some(redis_string[..size].to_string())},redis_string[size+2..].to_string()))
    }

    fn get_value(&self) -> Option<String>{
        match &self.content{
            Some(cont) => Some(cont.to_string()),
            None => None
       }
    }
}

impl RedisError{
    fn parse(mut redis_string: String) -> Result<(Self,String)>{
        redis_string = redis_string.trim_start_matches("-").to_string();
        let mut err_message = String::new();
        for (i, c) in redis_string.chars().enumerate(){
            if c.is_ascii_control() && match redis_string.chars().nth(i+1){
                Some(c) => c.is_ascii_control(),
                None => return Err(anyhow::Error::new(MyRedisError::InvalidRedisSynatx{message:format!("missing end of redis string (index: {} out of range for: {})",i.to_string(), redis_string)}))
            }{
                redis_string = redis_string[i+1..].to_string();
                break;
            }else{
                err_message.push(c);
            }
        }
        Ok((Self{error_msg: Some(err_message)},redis_string))
    }

    fn get_value(&self) -> Option<String>{
       match &self.error_msg{
            Some(err_msg) => Some(err_msg.to_string()),
            None => None
       }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::RedisType;

    #[test]
    fn test_redis_error_parse_correct(){
        let red_string = String::from("-ERROR: Can't convert\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["ERROR: Can't convert".to_string()]);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Inavlid Redis Syntax (unknown Redis Type char: ?)")]
    fn test_redis_error_invalid_typechar(){
        let red_string = String::from("?ERROR: Can't convert\r\n");
        RedisType::parse(red_string).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Inavlid Redis Syntax (missing end of redis string (index: 20 out of range for: ERROR: Can't convert\r))")]
    fn test_redis_error_missing_end_of_redis_string(){
        let red_string = String::from("-ERROR: Can't convert\r");
        RedisType::parse(red_string).unwrap();
    }

    #[test]
    fn test_redis_simple_string_parse_correct(){
        let red_string = String::from("+Hello World\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["Hello World".to_string()]);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Inavlid Redis Syntax (missing end of redis string (index: 11 out of range for: Hello World\r))")]
    fn test_redis_simple_string_missing_end_of_redis_string(){
        let red_string = String::from("+Hello World\r");
        RedisType::parse(red_string).unwrap();
    }

    #[test]
    fn test_redis_integer_parse_correct(){
        let red_string = String::from(":187\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["187".to_string()]);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Inavlid Redis Syntax (missing end of redis string (index: 3 out of range for: 187\r))")]
    fn test_redis_integer_missing_end_of_redis_string(){
        let red_string = String::from(":187\r");
        RedisType::parse(red_string).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: invalid digit found in string")]
    fn test_redis_integer_invalid_character(){
        let red_string = String::from(":13Hw\r\n");
        RedisType::parse(red_string).unwrap();
    }

    #[test]
    fn test_redis_bulk_string_parse_correct_1(){
        let red_string = String::from("$11\r\nHello World\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["Hello World".to_string()]);
    }

    #[test]
    fn test_redis_bulk_string_parse_correct_2(){
        let red_string = String::from("$15\r\nI love E girls!\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["I love E girls!".to_string()]);
    }

    #[test]
    fn test_redis_bulk_string_parse_correct_3(){
        let red_string = String::from("$17\r\nI love E girls!13\r\n");
        let resp = RedisType::parse(red_string).unwrap().unwrap();
        assert_eq!(resp.get_value().unwrap(),vec!["I love E girls!13".to_string()]);
    }
    
}