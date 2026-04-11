.PHONY: bench-overhead bench-matrix bench-modsim \
       bench-quick-light bench-quick-heavy \
       bench-quick-light-ab bench-quick-heavy-ab \
       bench-compare bench bench-full bench-finish

# Suites:
#   bench-quick-light  — 5 runners × 9 scenarios, 20 samples, ~3 min
#   bench-quick-heavy  — same runners/scenarios, 150 samples 40s measure, ~2-3 hours
#   bench-overhead     — fused vs handrolled (framework cost)
#   bench-matrix       — full 16-funnel × 14 scenarios + all baselines
#   bench-modsim       — module resolution simulation
#
# A/B comparison (multi-revision):
#   bench-quick-light-ab — quick-light across git revisions
#   bench-quick-heavy-ab — quick-heavy across git revisions (overnight)
#
# Compound:
#   bench-compare  — matrix + docs rebuild (daily driver)
#   bench          — matrix + modsim + docs
#   bench-full     — all + docs

# All paths absolute.
WS_ROOT := $(abspath $(CURDIR)/..)
SCRIPTS := $(WS_ROOT)/Makefile-scripting
BENCH_AB_ARCHIVE ?= $(WS_ROOT)/hylic/KB/.plans/funnel-gat-axes/bench-archive

# ── Quick (light): fast iteration ────────────────────
# Criterion params set here — the central control point for light runs.

bench-quick-light:
	@HYLIC_BENCH_SAMPLES=20 HYLIC_BENCH_WARMUP=1 HYLIC_BENCH_MEASURE=5 \
		bash $(SCRIPTS)/bench-one.sh bench_quick $(CURDIR)/target/bench-latest/quick

bench-quick-light-ab:
	@HYLIC_BENCH_SAMPLES=20 HYLIC_BENCH_WARMUP=1 HYLIC_BENCH_MEASURE=5 \
		bash $(SCRIPTS)/bench-ab.sh bench_quick \
		$(CURDIR)/target/bench-ab/quick-light \
		--archive $(BENCH_AB_ARCHIVE) \
		pre-arena=9565c7a \
		post-arena=9c83617 \
		post-clone-elim=HEAD

# ── Quick (heavy): overnight precision ───────────────
# Criterion params set here — the central control point for heavy runs.
# 80 samples / 20s measure: ~24s per cell.
#   bench-quick-heavy:    45 cells ≈ 18 min
#   bench-quick-heavy-ab: 3 revisions × 45 cells ≈ 54 min

bench-quick-heavy:
	@HYLIC_BENCH_SAMPLES=80 HYLIC_BENCH_WARMUP=2 HYLIC_BENCH_MEASURE=20 \
		bash $(SCRIPTS)/bench-one.sh bench_quick $(CURDIR)/target/bench-latest/quick-heavy

bench-quick-heavy-ab:
	@HYLIC_BENCH_SAMPLES=80 HYLIC_BENCH_WARMUP=2 HYLIC_BENCH_MEASURE=20 \
		bash $(SCRIPTS)/bench-ab.sh bench_quick \
		$(CURDIR)/target/bench-ab/quick-heavy \
		--archive $(BENCH_AB_ARCHIVE) \
		pre-arena=9565c7a \
		post-arena=9c83617 \
		post-clone-elim=HEAD

# ── Full suites ──────────────────────────────────────
# These use bench-one.sh defaults (80 samples, 4s warmup, 40s measure).
# Override via env: HYLIC_BENCH_SAMPLES=40 make bench-matrix

bench-overhead:
	@bash $(SCRIPTS)/bench-one.sh bench_overhead $(CURDIR)/target/bench-latest/overhead

bench-matrix:
	@bash $(SCRIPTS)/bench-one.sh bench_matrix $(CURDIR)/target/bench-latest/matrix

bench-modsim:
	@bash $(SCRIPTS)/bench-one.sh bench_modsim $(CURDIR)/target/bench-latest/modsim

bench-finish:
	@$(MAKE) -C $(WS_ROOT) docs-build

bench-compare: bench-matrix bench-finish

bench: bench-matrix bench-modsim bench-finish

bench-full: bench-overhead bench-matrix bench-modsim bench-quick-heavy
# bench-full: bench-overhead bench-matrix bench-modsim bench-quick-light bench-finish
