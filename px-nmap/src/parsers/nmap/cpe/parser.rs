use super::Error;


enum Token {
    // this is a name field (i/.../)
    Indentifer(Ident, String),
    CPE {
        product: String,
        os: String,
        info: String,
        devicetype: String,
        hostname: String
    },
    
}

enum Ident {
    Product,
    Version,
    Info,
    OperatingSystem,
    DeviceType,
    Hostname,
}

impl Ident {
    const IDENTS: [char; 6] = ['p', 'v', 'h', 'o', 'd', 'i'];

    pub fn is_ident_char(c: char) -> bool {
        Self::IDENTS.contains(&c)
    }

    pub fn identifier(&self) -> char {
        match self {
            Ident::Product         =>   'p',
            Ident::Version         =>   'v',
            Ident::Hostname        =>   'h',
            Ident::OperatingSystem =>   'o', 
            Ident::DeviceType      =>   'd',
            Ident::Info            =>   'i',
        }
    }

    pub fn from_identifier(c: char) -> Result<Self, Error> {
        Ok(match c {
            'p' => Ident::Product,
            'v' => Ident::Version,
            'h' => Ident::Hostname,
            'o' => Ident::OperatingSystem,
            'd' => Ident::DeviceType,
            'i' => Ident::Info,
            _ => return Err(Error::UnknownToken(c.to_string()))
        })
    }
}


fn step<T: Iterator<Item=char>>(mut line_buf: &str, ) -> Result<Vec<Token>, Error>{
    let mut stream = line_buf
        .chars()
        .filter_map(|c| match c { 
            ' ' | '\n' | '\t' => None,
            _ => Some(c)
        });
    
    let c = stream.next().ok_or_else(|| Error::ExpectedToken)?;

    if Ident::is_ident_char(c) {
        // construct Token::Identifer
        let ident = Ident::from_identifier(c)?;
        let delimiter = stream.next().ok_or_else(|| Error::ExpectedToken)?;
        
    }
    else {
        let mut temp: [u8; 3] = [0; 3];
        temp[0] = c as u8;

        for i in 1..3 {
            // 1 -> p,
            // 2 -> e
            temp[i] = stream.next().ok_or_else(|| Error::ExpectedToken)? as u8;
        }

        if b"cpe".eq(&temp) {
            let delimiter = stream.next().ok_or_else(|| Error::ExpectedToken)?;

        }

        else {
            
        }
    }

    Ok(unimplemented!())
}


struct CPESkeleton(String)

impl CPESkeleton {
    fn populate_enumerate(&self, &mut )
}

fn populate_expressions() -> Result<(), Error> {
    
    Ok(())
}