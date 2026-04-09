//! ExecutorSet: shared resources for all benchmark runners.

use std::sync::Arc;
use hylic::cata::exec::funnel;
use hylic::cata::exec::funnel::policy;
use hylic_parallel_lifts::WorkPool;

pub struct FunnelSpecs {
    pub pw_final:         funnel::Spec<policy::Default>,
    pub pw_arrive:        funnel::Spec<policy::PerWorkerArrival>,
    pub sh_final:         funnel::Spec<policy::SharedDefault>,
    pub sh_arrive:        funnel::Spec<policy::WideLight>,
    pub pw_final_batch:   funnel::Spec<policy::LowOverhead>,
    pub pw_final_k4:      funnel::Spec<policy::HighThroughput>,
    pub pw_final_k2:      funnel::Spec<policy::DeepNarrow>,
    pub sh_arrive_batch:  funnel::Spec<policy::StreamingWide>,
    pub lo:               funnel::Spec<policy::LowOverhead>,
}

impl FunnelSpecs {
    pub fn new(nw: usize) -> Self {
        use funnel::queue::per_worker::PerWorkerSpec;
        use funnel::queue::shared::SharedSpec;
        use funnel::accumulate::on_arrival::OnArrivalSpec;
        use funnel::accumulate::on_finalize::OnFinalizeSpec;
        use funnel::wake::every_push::EveryPushSpec;

        FunnelSpecs {
            pw_final:        funnel::Spec::default(nw),
            pw_arrive:       funnel::Spec::<policy::PerWorkerArrival>::new(nw, PerWorkerSpec { deque_capacity: 4096 }, OnArrivalSpec, EveryPushSpec),
            sh_final:        funnel::Spec::<policy::SharedDefault>::new(nw, SharedSpec, OnFinalizeSpec, EveryPushSpec),
            sh_arrive:       funnel::Spec::for_wide_light(nw),
            pw_final_batch:  funnel::Spec::for_low_overhead(nw),
            pw_final_k4:     funnel::Spec::for_high_throughput(nw),
            pw_final_k2:     funnel::Spec::for_deep_narrow(nw),
            sh_arrive_batch: funnel::Spec::for_streaming_wide(nw),
            lo:              funnel::Spec::for_low_overhead(nw),
        }
    }
}

/// Shared resources for a benchmark session. Constructed once, passed to all runners.
pub struct ExecutorSet<'a> {
    pub fpool: &'a funnel::Pool<'a>,
    pub wpool: &'a Arc<WorkPool>,
    pub nw: usize,
    pub sheque: hylic_benchmark::executor::hylo_sheque::Spec,
    pub funnel: FunnelSpecs,
}
