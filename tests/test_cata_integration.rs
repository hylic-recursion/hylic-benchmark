use hylic::domain::shared as dom;

#[derive(Clone)]
struct N { val: i32, children: Vec<N> }

#[test]
fn all_executors_match() {
    let tree = N { val: 1, children: vec![
        N { val: 2, children: vec![N { val: 4, children: vec![] }] },
        N { val: 3, children: vec![] },
    ]};
    let graph = hylic::graph::treeish(|n: &N| n.children.clone());
    let my_fold = dom::simple_fold(|n: &N| n.val as u64, |a: &mut u64, c: &u64| { *a += c; });

    assert_eq!(dom::FUSED.run(&my_fold, &graph, &tree), 10);
    assert_eq!(dom::exec(hylic_benchmark::executor::rayon::Spec).run(&my_fold, &graph, &tree), 10);
}

#[test]
fn all_executors_vec_fold() {
    #[derive(Clone)]
    struct T { name: String, children: Vec<T> }
    impl T {
        fn leaf(s: &str) -> Self { T { name: s.into(), children: vec![] } }
        fn branch(s: &str, ch: Vec<T>) -> Self { T { name: s.into(), children: ch } }
    }

    let tree = T::branch("a", vec![T::branch("b", vec![T::leaf("d"), T::leaf("e")]), T::leaf("c")]);
    let graph = hylic::graph::treeish(|n: &T| n.children.clone());
    use hylic::prelude::{vec_fold, VecHeap};
    let format = |heap: &VecHeap<T, String>| {
        let ch = heap.childresults.join(", ");
        if ch.is_empty() { heap.node.name.clone() } else { format!("{}[{}]", heap.node.name, ch) }
    };
    let my_fold = vec_fold(format);

    assert_eq!(dom::FUSED.run(&my_fold, &graph, &tree), "a[b[d, e], c]");
    assert_eq!(dom::exec(hylic_benchmark::executor::rayon::Spec).run(&my_fold, &graph, &tree), "a[b[d, e], c]");
}

