//! ExecutorSet: shared resources for all benchmark runners.

use std::sync::Arc;
use hylic::exec::funnel;
use hylic::exec::funnel::policy;
use hylic::exec::funnel::wake;
use hylic_parallel_lifts::WorkPool;

/// All 16 funnel policy variants: 4 queue×accumulate × 4 wake.
pub struct FunnelSpecs {
    // PerWorker + OnFinalize × 4 wake
    pub pw_fin:           funnel::Spec<policy::Default>,
    pub pw_fin_batch:     funnel::Spec<policy::LowOverhead>,
    pub pw_fin_k4:        funnel::Spec<policy::HighThroughput>,
    pub pw_fin_k2:        funnel::Spec<policy::DeepNarrow>,
    // PerWorker + OnArrival × 4 wake
    pub pw_arrv:          funnel::Spec<policy::PerWorkerArrival>,
    pub pw_arrv_batch:    funnel::Spec<policy::Policy<funnel::queue::PerWorker, funnel::accumulate::OnArrival, wake::OncePerBatch>>,
    pub pw_arrv_k4:       funnel::Spec<policy::Policy<funnel::queue::PerWorker, funnel::accumulate::OnArrival, wake::EveryK<4>>>,
    pub pw_arrv_k2:       funnel::Spec<policy::Policy<funnel::queue::PerWorker, funnel::accumulate::OnArrival, wake::EveryK<2>>>,
    // Shared + OnFinalize × 4 wake
    pub sh_fin:           funnel::Spec<policy::SharedDefault>,
    pub sh_fin_batch:     funnel::Spec<policy::Policy<funnel::queue::Shared, funnel::accumulate::OnFinalize, wake::OncePerBatch>>,
    pub sh_fin_k4:        funnel::Spec<policy::Policy<funnel::queue::Shared, funnel::accumulate::OnFinalize, wake::EveryK<4>>>,
    pub sh_fin_k2:        funnel::Spec<policy::Policy<funnel::queue::Shared, funnel::accumulate::OnFinalize, wake::EveryK<2>>>,
    // Shared + OnArrival × 4 wake
    pub sh_arrv:          funnel::Spec<policy::WideLight>,
    pub sh_arrv_batch:    funnel::Spec<policy::StreamingWide>,
    pub sh_arrv_k4:       funnel::Spec<policy::Policy<funnel::queue::Shared, funnel::accumulate::OnArrival, wake::EveryK<4>>>,
    pub sh_arrv_k2:       funnel::Spec<policy::Policy<funnel::queue::Shared, funnel::accumulate::OnArrival, wake::EveryK<2>>>,
}

// ANCHOR: funnel_specs
impl FunnelSpecs {
    pub fn new(nw: usize) -> Self {
        use funnel::wake::once_per_batch::OncePerBatchSpec;
        use funnel::wake::every_k::EveryKSpec;

        FunnelSpecs {
            // PW + Final × wake
            pw_fin:         funnel::Spec::default(nw),
            pw_fin_batch:   funnel::Spec::for_low_overhead(nw),
            pw_fin_k4:      funnel::Spec::for_high_throughput(nw),
            pw_fin_k2:      funnel::Spec::for_deep_narrow(nw),
            // PW + Arrive × wake
            pw_arrv:        funnel::Spec::for_perworker_arrival(nw),
            pw_arrv_batch:  funnel::Spec::for_perworker_arrival(nw).with_wake::<wake::OncePerBatch>(OncePerBatchSpec),
            pw_arrv_k4:     funnel::Spec::for_perworker_arrival(nw).with_wake::<wake::EveryK<4>>(EveryKSpec),
            pw_arrv_k2:     funnel::Spec::for_perworker_arrival(nw).with_wake::<wake::EveryK<2>>(EveryKSpec),
            // SH + Final × wake
            sh_fin:         funnel::Spec::for_shared_default(nw),
            sh_fin_batch:   funnel::Spec::for_shared_default(nw).with_wake::<wake::OncePerBatch>(OncePerBatchSpec),
            sh_fin_k4:      funnel::Spec::for_shared_default(nw).with_wake::<wake::EveryK<4>>(EveryKSpec),
            sh_fin_k2:      funnel::Spec::for_shared_default(nw).with_wake::<wake::EveryK<2>>(EveryKSpec),
            // SH + Arrive × wake
            sh_arrv:        funnel::Spec::for_wide_light(nw),
            sh_arrv_batch:  funnel::Spec::for_streaming_wide(nw),
            sh_arrv_k4:     funnel::Spec::for_wide_light(nw).with_wake::<wake::EveryK<4>>(EveryKSpec),
            sh_arrv_k2:     funnel::Spec::for_wide_light(nw).with_wake::<wake::EveryK<2>>(EveryKSpec),
        }
    }
}
// ANCHOR_END: funnel_specs

/// Shared resources for a benchmark session. Constructed once, passed to all runners.
pub struct ExecutorSet<'a> {
    pub fpool: &'a funnel::Pool<'a>,
    pub wpool: &'a Arc<WorkPool>,
    pub nw: usize,
    pub sheque: hylic_benchmark::executor::hylo_sheque::Spec,
    pub funnel: FunnelSpecs,
}
