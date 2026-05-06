.PHONY: bench-overhead bench-matrix bench-modsim \
       bench-quick-light bench-quick-heavy \
       bench-quick-light-ab bench-quick-heavy-ab \
       bench-publish bench-compare bench bench-full

# Suites:
#   bench-quick-light  — 5 runners × 9 scenarios, 20 samples, ~3 min
#   bench-quick-heavy  — same runners/scenarios, 80 samples 20s measure, ~18 min
#   bench-overhead     — fused vs handrolled (framework cost)
#   bench-matrix       — full 16-funnel × 14 scenarios + all baselines
#   bench-modsim       — module resolution simulation
#
# A/B comparison (multi-revision):
#   bench-quick-light-ab — quick-light across git revisions of hylic
#   bench-quick-heavy-ab — quick-heavy across git revisions (overnight)
#
# Compound:
#   bench-compare  — matrix + publish to hylic-docs
#   bench          — matrix + modsim + publish
#   bench-full     — all benches (publish separately afterwards if desired)
#
# Publication to hylic-docs/book/src/bench-results/ is an explicit step
# (`make bench-publish`); it requires the workspace or CI layout. The
# rendered docs site picks up the committed bench-results files from
# hylic-docs's master branch, so doc rebuild is always a separate
# operation owned by hylic-docs (`make -C hylic-docs hylic-docs-build`).

SCRIPTS := $(CURDIR)/scripts

# ── Quick (light): fast iteration ────────────────────
bench-quick-light:
	@HYLIC_BENCH_SAMPLES=20 HYLIC_BENCH_WARMUP=1 HYLIC_BENCH_MEASURE=5 \
		bash $(SCRIPTS)/bench-one.bash bench_quick $(CURDIR)/target/bench-latest/quick

bench-quick-light-ab:
	@HYLIC_BENCH_SAMPLES=20 HYLIC_BENCH_WARMUP=1 HYLIC_BENCH_MEASURE=5 \
		bash $(SCRIPTS)/bench-ab.bash bench_quick \
		$(CURDIR)/target/bench-ab/quick-light \
		$(if $(BENCH_AB_ARCHIVE),--archive $(BENCH_AB_ARCHIVE)) \
		pre-arena=9565c7a \
		post-arena=9c83617 \
		post-clone-elim=HEAD

# ── Quick (heavy): overnight precision ───────────────
# 80 samples / 20s measure: ~24s per cell.
#   bench-quick-heavy:    45 cells ≈ 18 min
#   bench-quick-heavy-ab: 3 revisions × 45 cells ≈ 54 min
bench-quick-heavy:
	@HYLIC_BENCH_SAMPLES=80 HYLIC_BENCH_WARMUP=2 HYLIC_BENCH_MEASURE=20 \
		bash $(SCRIPTS)/bench-one.bash bench_quick $(CURDIR)/target/bench-latest/quick-heavy

bench-quick-heavy-ab:
	@HYLIC_BENCH_SAMPLES=80 HYLIC_BENCH_WARMUP=2 HYLIC_BENCH_MEASURE=20 \
		bash $(SCRIPTS)/bench-ab.bash bench_quick \
		$(CURDIR)/target/bench-ab/quick-heavy \
		$(if $(BENCH_AB_ARCHIVE),--archive $(BENCH_AB_ARCHIVE)) \
		pre-arena=9565c7a \
		post-arena=9c83617 \
		post-clone-elim=HEAD

# ── Full suites ──────────────────────────────────────
# Use bench-one.bash defaults (80 samples, 4s warmup, 40s measure).
# Override via env: HYLIC_BENCH_SAMPLES=40 make bench-matrix
bench-overhead:
	@bash $(SCRIPTS)/bench-one.bash bench_overhead $(CURDIR)/target/bench-latest/overhead

bench-matrix:
	@bash $(SCRIPTS)/bench-one.bash bench_matrix $(CURDIR)/target/bench-latest/matrix

bench-modsim:
	@bash $(SCRIPTS)/bench-one.bash bench_modsim $(CURDIR)/target/bench-latest/modsim

# ── Publication (workspace / CI mode only) ───────────
bench-publish:
	@bash $(SCRIPTS)/publish-to-docs.bash

bench-compare: bench-matrix bench-publish

bench: bench-matrix bench-modsim bench-publish

bench-full: bench-overhead bench-matrix bench-modsim bench-quick-heavy
