use px_nmap::service_probe::parser::parse;
use tokio::runtime::Builder;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct CLI {
    #[structopt(parse(from_os_str))]
    db: Option<PathBuf>
}

fn main() {
    let args: CLI = CLI::from_args();

    let rt = Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    
    let path = args.db.unwrap_or("/usr/share/nmap/nmap-service-probes".parse().unwrap());

    if !path.is_file() {
        eprintln!("'{}' Isn't a file", path.to_string_lossy());
        return
    }
    println!("reading path '{}'", path.to_str().unwrap());
    
    
    rt.block_on(async move {
        let mut buf = Vec::with_capacity(220);

        if let Err(why) = parse(&path.to_string_lossy(), &mut buf).await {
            println!("{:#?}", why);
            return
        }
        
        //dbg!(buf);
    })
}