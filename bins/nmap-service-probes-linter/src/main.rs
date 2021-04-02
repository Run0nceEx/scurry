use px_nmap::service_probe::parser::parse;
use tokio::runtime::Builder;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct CLI {
    #[structopt(parse(from_os_str))]
    db: Option<PathBuf>,
    
    #[structopt(short)]
    out: bool
}

fn main() {
    let args: CLI = CLI::from_args();

    let rt = Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    
    let path = args.db.unwrap_or("/usr/share/nmap/nmap-service-probes".parse().unwrap());
    let print_flag = args.out;
    
    if !path.is_file() {
        eprintln!("'{}' Isn't a file", path.to_string_lossy());
        return
    }
    eprintln!("reading path '{}'", path.to_str().unwrap());
    
    
    rt.block_on(async move {
        let mut buf = Vec::with_capacity(220);

        if let Err(why) = parse(&path.to_string_lossy(), &mut buf).await {
            dbg!(buf);
            eprintln!("{}", &why.error);
            eprintln!("col: {}, span: {}..{}", &why.cursor.col, &why.cursor.span.start, &why.cursor.span.end);
            return
        }
        if !print_flag {
            eprintln!("OK!")
        }    
        else { println!("{:#?}", buf); } 
    })
}