
error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Utf8(::std::string::FromUtf8Error);
    }

    errors { 
        UnexpectedEndOfInput {
            description("Encountered unexpected end of input while reading message from server.")
            display("Encountered unexpected end of input while reading message from server.")
        }

        InputTooLong(message: String) {
            description("The input was too long.")
            display("{}", message)
        }
        
        ConnectionReset {
            description("The connection was reset by the remote host.")
            display("The connection was reset by the remote host.")
        }
    }
    
}
