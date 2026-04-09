.PHONY: bench-compare bench bench-full bench-finish \
       bench-variants bench-variants-reset

# ── Benchmarks ──────────────────────────────────────────────
# Each bench target runs criterion, generates report, rebuilds docs.
#
# bench-compare: funnel vs hylo vs rayon (daily driver)
# bench:         parallel + comparative
# bench-full:    everything

# Atomic bench units (not user-facing)
# Each calls bench-one.sh -> streams output, archives raw + criterion + report
_bench-seq:
	@bash ../Makefile-scripting/bench-one.sh bench_sequential target/bench-latest/sequential
_bench-par:
	@bash ../Makefile-scripting/bench-one.sh bench_parallel target/bench-latest/parallel
_bench-module:
	@bash ../Makefile-scripting/bench-one.sh bench_module_sim target/bench-latest/module-sim
_bench-executor:
	@bash ../Makefile-scripting/bench-one.sh bench_executor_compare target/bench-latest/executor-compare
_bench-hylo:
	@bash ../Makefile-scripting/bench-one.sh bench_hylo_compare target/bench-latest/hylo-compare

# Copy reports to docs + rebuild book
bench-finish:
	@for d in target/bench-latest/*/report; do \
		cp -r "$$d"/* ../hylic-docs/book/src/bench-results/ 2>/dev/null || true; \
	done
	@cd ../hylic-docs/book && mdbook build

# User-facing targets
bench-compare: _bench-hylo bench-finish

bench: _bench-par _bench-hylo bench-finish

bench-full: _bench-seq _bench-par _bench-module _bench-executor _bench-hylo bench-finish

# Cross-variant comparison (git tags, same bench, different source)
bench-variants:
	@bash _bench-experiment/run-all.sh

# Restore hylic src/ to current HEAD after interrupted bench-variants run
bench-variants-reset:
	@cd ../hylic && git checkout HEAD -- src/
	@echo "hylic src/ restored to HEAD"
