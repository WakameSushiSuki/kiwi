use kiwi::text::token::{Cursor, Token};

fn main(){
    let src = include_str!("../../examples/time.kw");
    let mut cur = Cursor::new(src);
    loop{
        cur.take_while(|x| x.is_whitespace());
        match cur.read_token::<Token>() {
            Ok(t) => println!("{t:?}"),
            Err(()) => {println!("ERR"); break;}
        }
    }
}