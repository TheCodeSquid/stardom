// TODO: expand with more tests
// - panic assertions for invalid operations

#[macro_export]
macro_rules! behavior_tests {
    ($node_ty:ty) => {{
        type N = $node_ty;
        use stardom_nodes::*;
        stardom_reactive::Runtime::new().init();

        let a = N::text();
        let b = N::text();
        let root = div! { a; b; };

        assert_eq!(root.children(), vec![a.clone(), b.clone()]);
        assert_eq!(a.next_sibling(), Some(b.clone()));

        let c = N::text();
        let d = N::text();
        let frag = fragment! { c; d; };
        root.insert(&frag, Some(&b));

        assert_eq!(a.next_sibling(), Some(frag.clone()));
        assert_eq!(frag.next_sibling(), Some(b.clone()));
        assert_eq!(c.next_sibling(), Some(d.clone()));
        assert_eq!(d.next_sibling(), None);

        root.insert(&frag, None);
        assert_eq!(b.next_sibling(), Some(frag.clone()));
        assert_eq!(c.next_sibling(), Some(d.clone()));
    }};
}
