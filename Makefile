.PHONY: bench-overhead bench-matrix bench-modsim \
       bench-compare bench bench-full bench-finish

# Three suites, three targets. No indirection.
#
#   bench-overhead — fused vs handrolled (framework cost)
#   bench-matrix   — full 16-funnel × 14 scenarios + all baselines
#   bench-modsim   — module resolution simulation
#
# Compound:
#   bench-compare  — matrix + docs rebuild (daily driver)
#   bench          — matrix + modsim + docs
#   bench-full     — all three + docs

bench-overhead:
	@bash ../Makefile-scripting/bench-one.sh bench_overhead target/bench-latest/overhead

bench-matrix:
	@bash ../Makefile-scripting/bench-one.sh bench_matrix target/bench-latest/matrix

bench-modsim:
	@bash ../Makefile-scripting/bench-one.sh bench_modsim target/bench-latest/modsim

bench-finish:
	@cd ../hylic-docs/book && mdbook build

bench-compare: bench-matrix bench-finish

bench: bench-matrix bench-modsim bench-finish

bench-full: bench-overhead bench-matrix bench-modsim bench-finish
