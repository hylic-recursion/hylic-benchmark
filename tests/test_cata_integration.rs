use hylic::domain::shared as dom;
use hylic::domain::local;
use hylic_parallel_lifts::{WorkPool, WorkPoolSpec};

fn n_threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
}

#[derive(Clone)]
struct N { val: i32, children: Vec<N> }

#[test]
fn all_executors_match() {
    let tree = N { val: 1, children: vec![
        N { val: 2, children: vec![N { val: 4, children: vec![] }] },
        N { val: 3, children: vec![] },
    ]};
    let graph = dom::treeish(|n: &N| n.children.clone());
    let my_fold = dom::simple_fold(|n: &N| n.val as u64, |a: &mut u64, c: &u64| { *a += c; });

    assert_eq!(dom::FUSED.run(&my_fold, &graph, &tree), 10);
    assert_eq!(hylic_benchmark::statics::RAYON.run(&my_fold, &graph, &tree), 10);
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
    let graph = dom::treeish(|n: &T| n.children.clone());
    use hylic::prelude::{vec_fold, VecHeap};
    let format = |heap: &VecHeap<T, String>| {
        let ch = heap.childresults.join(", ");
        if ch.is_empty() { heap.node.name.clone() } else { format!("{}[{}]", heap.node.name, ch) }
    };
    let my_fold = vec_fold(format);

    assert_eq!(dom::FUSED.run(&my_fold, &graph, &tree), "a[b[d, e], c]");
    assert_eq!(hylic_benchmark::statics::RAYON.run(&my_fold, &graph, &tree), "a[b[d, e], c]");
}

#[test]
fn parallel_lifts() {
    let tree = N { val: 1, children: vec![
        N { val: 2, children: vec![N { val: 4, children: vec![] }] },
        N { val: 3, children: vec![] },
    ]};
    let graph = dom::treeish(|n: &N| n.children.clone());
    let my_fold = dom::simple_fold(|n: &N| n.val as u64, |a: &mut u64, c: &u64| { *a += c; });

    use hylic_parallel_lifts::{ParLazy, ParEager, EagerSpec};
    let nt = n_threads();
    WorkPool::with(WorkPoolSpec::threads(nt), |pool| {
        assert_eq!(dom::FUSED.run_lifted(&ParLazy::lift(pool), &my_fold, &graph, &tree), 10);
        assert_eq!(dom::FUSED.run_lifted(&ParEager::lift(pool, EagerSpec::default_for(nt)), &my_fold, &graph, &tree), 10);
    });
}

fn big_tree(n: usize, bf: usize) -> N {
    fn build(id: &mut i32, remaining: &mut usize, bf: usize) -> N {
        let val = *id; *id += 1; *remaining = remaining.saturating_sub(1);
        let mut children = Vec::new();
        for _ in 0..bf { if *remaining == 0 { break; } children.push(build(id, remaining, bf)); }
        N { val, children }
    }
    let mut id = 1; let mut remaining = n;
    build(&mut id, &mut remaining, bf)
}

#[test]
fn lifts_domain_generic_comprehensive() {
    use hylic_parallel_lifts::{ParLazy, ParEager, EagerSpec};

    let tree = big_tree(60, 4);
    let shared_fold = dom::simple_fold(|n: &N| n.val, |a: &mut i32, c: &i32| { *a += c; });
    let shared_graph = dom::treeish(|n: &N| n.children.clone());
    let expected = dom::FUSED.run(&shared_fold, &shared_graph, &tree);

    let make_local_fold = || local::fold(
        |n: &N| n.val, |a: &mut i32, c: &i32| { *a += c; }, |h: &i32| *h);
    let make_local_graph = || local::treeish_visit(|n: &N, cb: &mut dyn FnMut(&N)| {
        for c in &n.children { cb(c); }
    });

    let nt = n_threads();
    WorkPool::with(WorkPoolSpec::threads(nt), |pool| {
        // Shared: all executor x lift combos
        assert_eq!(dom::FUSED.run_lifted(&ParLazy::lift(pool), &shared_fold, &shared_graph, &tree), expected, "Lazy+Fused+Shared");
        assert_eq!(hylic_benchmark::statics::RAYON.run_lifted(&ParLazy::lift(pool), &shared_fold, &shared_graph, &tree), expected, "Lazy+Rayon+Shared");

        assert_eq!(dom::FUSED.run_lifted(&ParEager::lift(pool, EagerSpec::default_for(nt)), &shared_fold, &shared_graph, &tree), expected, "Eager+Fused+Shared");
        assert_eq!(hylic_benchmark::statics::RAYON.run_lifted(&ParEager::lift(pool, EagerSpec::default_for(nt)), &shared_fold, &shared_graph, &tree), expected, "Eager+Rayon+Shared");

        // Local: Fused x lift combos
        let (lf, lg) = (make_local_fold(), make_local_graph());
        assert_eq!(local::FUSED.run_lifted(&ParLazy::lift(pool), &lf, &lg, &tree), expected, "Lazy+Fused+Local");
        let (lf, lg) = (make_local_fold(), make_local_graph());
        assert_eq!(local::FUSED.run_lifted(&ParEager::lift(pool, EagerSpec::default_for(nt)), &lf, &lg, &tree), expected, "Eager+Fused+Local");
    });
}

/// Level 6 stress: ParEager on a 200-node tree, 4 workers, 20 iterations.
#[test]
fn stress_eager_200_nodes() {
    use hylic_parallel_lifts::{ParEager, EagerSpec};

    let tree = big_tree(200, 6);
    let fold = dom::simple_fold(|n: &N| n.val, |a: &mut i32, c: &i32| { *a += c; });
    let graph = dom::treeish(|n: &N| n.children.clone());
    let expected = dom::FUSED.run(&fold, &graph, &tree);

    let nt = n_threads();
    for iteration in 0..20 {
        WorkPool::with(WorkPoolSpec::threads(nt), |pool| {
            let result = dom::FUSED.run_lifted(
                &ParEager::lift(pool, EagerSpec::default_for(nt)),
                &fold, &graph, &tree,
            );
            assert_eq!(result, expected, "eager stress iteration {iteration}");
        });
    }
}

/// Level 6 stress: ParLazy same parameters.
#[test]
fn stress_lazy_200_nodes() {
    use hylic_parallel_lifts::ParLazy;

    let tree = big_tree(200, 6);
    let fold = dom::simple_fold(|n: &N| n.val, |a: &mut i32, c: &i32| { *a += c; });
    let graph = dom::treeish(|n: &N| n.children.clone());
    let expected = dom::FUSED.run(&fold, &graph, &tree);

    let nt = n_threads();
    for iteration in 0..20 {
        WorkPool::with(WorkPoolSpec::threads(nt), |pool| {
            let result = dom::FUSED.run_lifted(
                &ParLazy::lift(pool),
                &fold, &graph, &tree,
            );
            assert_eq!(result, expected, "lazy stress iteration {iteration}");
        });
    }
}
