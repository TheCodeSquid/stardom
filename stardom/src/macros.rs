#[macro_export]
macro_rules! clone {
    ($($var:ident),* ; $f:expr) => {
        {
            $(let mut $var = ::std::clone::Clone::clone(&$var);)*
            $f
        }
    };
}

#[macro_export]
macro_rules! effect {
    ($($($var:ident),* ;)? { $($body:tt)* }) => {
        {
            $($(let mut $var = ::std::clone::Clone::clone(&$var);)*)?
            stardom::effect(move || { $($body)* });
        }
    };
}

#[macro_export]
macro_rules! memo {
    ($($($var:ident),* ;)? { $($body:tt)* }) => {
        {
            $($(let mut $var = ::std::clone::Clone::clone(&$var);)*)?
            stardom::memo(move || { $($body)* })
        }
    };
}
