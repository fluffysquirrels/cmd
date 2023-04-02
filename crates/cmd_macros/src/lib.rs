#[macro_use]
mod lazy_regex;

#[macro_export]
macro_rules! cmd {
    (@word $id:ident) => {
        stringify!($id)
    };

    (@word $lit:literal) => { $lit };

    (@dcmd $bin:tt ) => {
        ::duct::cmd( $crate::cmd!(@word $bin), [] as [&'static str; 0] )
    };

    (@dcmd $bin:tt $( $arg:tt )+ ) => {
        ::duct::cmd( $crate::cmd!(@word $bin), [ $( $crate::cmd!(@word $arg) ),* ] )
    };

    (@out_redirect $out:tt ) => {
        $crate::cmd!( @word $out )
    };

    (@in_redirect $in:tt ) => {
        $crate::cmd!( @word $in )
    };

    (@redirect $expr:ident) => { $expr };

    (@redirect $expr:ident | $( $right:tt )+ ) => {
        $expr.pipe( $crate::cmd!( $( $right )+ ) )
    };

    (@redirect $expr:ident <<< $var:ident $( $rest:tt )* ) => {{
        let e = $expr.stdin_bytes($var);
        $crate::cmd!( @redirect e $( $rest )* )
    }};

    (@redirect $expr:ident < null $( $rest:tt )* ) => {{
        let e = $expr.stdin_null();
        $crate::cmd!( @redirect e $( $rest )* )
    }};

    (@redirect $expr:ident < $in:tt $( $rest:tt )* ) => {{
        let e = $expr.stdin_path( $crate::cmd!( @in_redirect $in ) );
        $crate::cmd!( @redirect e $( $rest )* )
    }};

    (@redirect $expr:ident > null $( $rest:tt )* ) => {{
        let e = $expr.stdout_null();
        $crate::cmd!( @redirect e $( $rest )* )
    }};

    (@redirect $expr:ident > $out:tt $( $rest:tt )* ) => {{
        let e = $expr.stdout_path( $crate::cmd!( @out_redirect $out ) );
        $crate::cmd!( @redirect e $( $rest )* )
    }};

    ( ( $( $left:tt )+ ) | $( $right:tt )+ ) => {
        $crate::cmd!( @dcmd $( $left )+ )
            .pipe( $crate::cmd!( $( $right )+ ) )
    };

    ( ( $( $left:tt )+ ) $( $redirect:tt )+ ) => {{
        let e = $crate::cmd!( @dcmd $( $left )+ );
        $crate::cmd!( @redirect e $( $redirect )+ )
    }};

    ( ( $( $left:tt )+ ) ) => {
        $crate::cmd!( @dcmd $( $left )+ )
    };

    ( $bin:ident $( $args:tt )* ) => {
        $crate::cmd!( @dcmd $bin $( $args )* )
    };

    ( $bin:literal $( $args:tt )* ) => {
        $crate::cmd!( @dcmd $bin $( $args )* )
    };
}

#[cfg(test)]
mod tests {
    use crate::cmd;

    #[test]
    fn first() {
        let _expr: duct::Expression = duct::cmd("bin", ["arg1", "arg2", "arg3"]);


        fn case(expr: duct::Expression, d: &str) {
            let d_trimmed = lazy_regex!("\n *").replace_all(d, "");
            assert_eq!(format!("{expr:?}"), &*d_trimmed);
        }

        let _expr: duct::Expression = cmd!(bin);
        let _expr: duct::Expression = cmd!("bin");
        let _expr: duct::Expression = cmd!(bin arg1 arg2 arg3);
        let _expr: duct::Expression = cmd!(bin "arg1a arg1b" arg2);
        let _expr: duct::Expression = cmd!("bin" arg1);
        let _expr: duct::Expression = cmd!("bin" "arg1");

        case(cmd!((bin)),
r#"
Cmd(["bin"])
"#);

        case(cmd!((bin1 arg11) | bin2 arg21),
r#"
Pipe(Cmd(["bin1", "arg11"]), 
     Cmd(["bin2", "arg21"])
)"#);

        case(cmd!((bin1 arg11) | (bin2 arg21) | bin3 arg31),
r#"
Pipe(Cmd(["bin1", "arg11"]), 
     Pipe(Cmd(["bin2", "arg21"]), 
          Cmd(["bin3", "arg31"])
     )
)"#);


        case(cmd!((bin arg1) > file),
r#"
Io(StdoutPath("file"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) > null),
r#"
Io(StdoutNull, 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) > "null"),
r#"
Io(StdoutPath("null"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) < file),
r#"
Io(StdinPath("file"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) < "null"),
r#"
Io(StdinPath("null"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) < null),
r#"
Io(StdinNull, 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!((bin arg1) < file_in > file_out),
r#"
Io(StdoutPath("file_out"), 
   Io(StdinPath("file_in"), 
      Cmd(["bin", "arg1"])
   )
)"#);

        case(cmd!((bin arg1) > file_out < file_in),
r#"
Io(StdinPath("file_in"), 
   Io(StdoutPath("file_out"), 
      Cmd(["bin", "arg1"])
   )
)"#);


        case(cmd!((bin1 arg1) < file_in | bin2),
r#"
Pipe(Io(StdinPath("file_in"), 
        Cmd(["bin1", "arg1"])
     ), 
     Cmd(["bin2"])
)"#);

        let in_bytes = "abc";

        case(cmd!((bin arg1) <<< in_bytes),
r#"
Io(StdinBytes([97, 98, 99]), 
   Cmd(["bin", "arg1"])
)"#);

    }
}
