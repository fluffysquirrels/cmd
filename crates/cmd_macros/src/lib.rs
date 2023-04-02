#[macro_use]
mod lazy_regex;

#[macro_export]
macro_rules! cmd {
    // In the external interface $bin is matched against ident or literal.
    // It might seem like we could match $bin:tt instead, because ( @bin $bin )
    // will check the token type, but then $bin:tt will match all the internal
    // patterns with @-prefixes.

    ( $bin:ident $( $rest:tt )* ) => {
        $crate::cmd!( @cmd_muncher ( ( $crate::cmd!( @bin $bin ) ) )
                                   ( $( $rest )* ) )
    };

    ( $bin:literal $( $rest:tt )* ) => {
        $crate::cmd!( @cmd_muncher ( ( $crate::cmd!( @bin $bin ) ) )
                                   ( $( $rest )* ) )
    };

    // ( @cmd_muncher ( $( $wip:tt )+ ) ( $( $rest:tt )* ) ) -> expr<duct::Expression>

    (@cmd_muncher ( $( $wip:tt )+ ) ( ) ) => {
        $crate::cmd!( @dcmd $( $wip )+ )
    };

    (@cmd_muncher ( $( $wip:tt )+ ) ( | $( $rest:tt )+ ) ) => {
        $crate::cmd!( @dcmd $( $wip )+ )
            .pipe( $crate::cmd!( $( $rest )+ ) )
    };

    (@cmd_muncher ( $( $wip:tt )+ ) ( < $( $rest:tt )+ ) ) => {
        $crate::cmd!( @opt_muncher ( $crate::cmd!( @dcmd $( $wip )+ ) )
                                   ( < $( $rest )* )
        )
    };

    (@cmd_muncher ( $( $wip:tt )+ ) ( > $( $rest:tt )+ ) ) => {
        $crate::cmd!( @opt_muncher ( $crate::cmd!( @dcmd $( $wip )+ ) )
                                   ( > $( $rest )* )
        )
    };

    (@cmd_muncher ( $( $wip:tt )+ ) ( <<< $( $rest:tt )+ ) ) => {
        $crate::cmd!( @opt_muncher ( $crate::cmd!( @dcmd $( $wip )+ ) )
                                   ( <<< $( $rest )* )
        )
    };

    (@cmd_muncher ( $( $wip:tt )+ ) ( $arg:tt $( $rest:tt )* ) ) => {
        $crate::cmd!( @cmd_muncher ( $( $wip )+
                                     ( $crate::cmd!( @arg $arg ) )
                                   )
                                   ( $( $rest )* )
        )
    };

    // ( @opt_muncher ( $wip:expr ) ( $( $opts:tt )* ) ) -> expr<duct::Expression>

    (@opt_muncher ( $wip:expr ) ( ) ) => {
        $wip
    };

    (@opt_muncher ( $wip:expr ) ( < null $( $rest:tt )* ) ) => {
        $crate::cmd!( @opt_muncher (
                          $wip.stdin_null()
                      )
                      ( $( $rest )* )
        )
    };

    (@opt_muncher ( $wip:expr ) ( < $path:tt $( $rest:tt )* ) ) => {
        $crate::cmd!( @opt_muncher (
                          $wip.stdin_path( $crate::cmd!( @path $path ) )
                      )
                      ( $( $rest )* )
        )
    };

    (@opt_muncher ( $wip:expr ) ( > null $( $rest:tt )* ) ) => {
        $crate::cmd!( @opt_muncher (
                          $wip.stdout_null()
                      )
                      ( $( $rest )* )
        )
    };

    (@opt_muncher ( $wip:expr ) ( > $path:tt $( $rest:tt )* ) ) => {
        $crate::cmd!( @opt_muncher (
                          $wip.stdout_path( $crate::cmd!( @path $path ) )
                      )
                      ( $( $rest )* )
        )
    };

    (@opt_muncher ( $wip:expr ) ( <<< $in:ident $( $rest:tt )* ) ) => {
        $crate::cmd!( @opt_muncher (
                          $wip.stdin_bytes( $in )
                      )
                      ( $( $rest )* )
        )
    };

    (@opt_muncher ( $wip:expr ) ( | $( $rest:tt )+ ) ) => {
        $wip.pipe( $crate::cmd!( $( $rest )+ ) )
    };

    // end @opt_muncher

    (@dcmd $bin:tt ) => {
        ::duct::cmd( $bin, [] as [&'static str; 0] )
    };

    (@dcmd $bin:tt $( $arg:tt )+ ) => {
        ::duct::cmd( $bin, [ $( $arg ),* ] )
    };

    (@arg $arg:ident) => {
        $crate::cmd!( @word $arg )
    };

    (@arg $arg:literal) => {
        $crate::cmd!( @word $arg )
    };

    (@path $path:ident) => {
        $crate::cmd!( @word $path )
    };

    (@path $path:literal) => {
        $crate::cmd!( @word $path )
    };

    (@bin $bin:ident) => {
        $crate::cmd!( @word $bin )
    };

    (@bin $bin:literal) => {
        $crate::cmd!( @word $bin )
    };

    (@word $id:ident) => {
        stringify!($id)
    };

    (@word $lit:literal) => { $lit };
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

        case(cmd!(bin),
             r#"
                 Cmd(["bin"])
             "#);
        case(cmd!("bin"),
             r#"
                 Cmd(["bin"])
             "#);
        case(cmd!(bin arg1 arg2 arg3),
             r#"
                 Cmd(["bin", "arg1", "arg2", "arg3"])
             "#);
        case(cmd!(bin "arg1a arg1b" arg2),
             r#"
                 Cmd(["bin", "arg1a arg1b", "arg2"])
             "#);
        case(cmd!("bin" arg1),
             r#"
                 Cmd(["bin", "arg1"])
             "#);
        case(cmd!("bin" "arg1"),
             r#"
                 Cmd(["bin", "arg1"])
             "#);

        case(cmd!(bin1 arg11 | bin2 arg21),
             r#"
                 Pipe(Cmd(["bin1", "arg11"]), 
                      Cmd(["bin2", "arg21"])
                 )
             "#);

        case(cmd!(bin1 arg11 | bin2 arg21 | bin3 arg31),
            r#"
                Pipe(Cmd(["bin1", "arg11"]), 
                     Pipe(Cmd(["bin2", "arg21"]), 
                          Cmd(["bin3", "arg31"])
                     )
                )
            "#);


        case(cmd!(bin arg1 > file),
r#"
Io(StdoutPath("file"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 > null),
r#"
Io(StdoutNull, 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 > "null"),
r#"
Io(StdoutPath("null"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 < file),
r#"
Io(StdinPath("file"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 < "null"),
r#"
Io(StdinPath("null"), 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 < null),
r#"
Io(StdinNull, 
   Cmd(["bin", "arg1"])
)"#);

        case(cmd!(bin arg1 < file_in > file_out),
r#"
Io(StdoutPath("file_out"), 
   Io(StdinPath("file_in"), 
      Cmd(["bin", "arg1"])
   )
)"#);

        case(cmd!(bin arg1 > file_out < file_in),
r#"
Io(StdinPath("file_in"), 
   Io(StdoutPath("file_out"), 
      Cmd(["bin", "arg1"])
   )
)"#);


        case(cmd!(bin1 arg1 < file_in | bin2),
r#"
Pipe(Io(StdinPath("file_in"), 
        Cmd(["bin1", "arg1"])
     ), 
     Cmd(["bin2"])
)"#);

        let in_bytes = "abc";

        case(cmd!(bin arg1 <<< in_bytes),
r#"
Io(StdinBytes([97, 98, 99]), 
   Cmd(["bin", "arg1"])
)"#);

    }
}
