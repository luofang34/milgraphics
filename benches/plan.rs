//! Construction and rendering throughput at 100, 1,000 and 10,000 graphics,
//! cycling through the six first-milestone symbols over a 1° × 1° area.
//!
//! Criterion is a native-only dev-dependency; on wasm32 (which CI lints with
//! `--all-targets`) the benchmark compiles to an empty program.

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use criterion::{BenchmarkId, Criterion, criterion_group};
    use milgraphics::render::{FixedAdvanceMetrics, LocalEquirectangular};
    use milgraphics::{
        Config, Construction, ControlPoint, GeoPoint, GraphicDefinition, GraphicId, ModifierField,
        ModifierValue, SymbolId, View, construct, render,
    };
    use std::hint::black_box;

    const SIZES: [usize; 3] = [100, 1_000, 10_000];

    /// The `i`-th graphic: one of the six symbols, offset on a grid.
    fn graphic(i: usize) -> Option<GraphicDefinition> {
        let (lon0, lat0) = (
            20.0 + (i % 100) as f64 * 0.01,
            50.0 + (i / 100 % 100) as f64 * 0.01,
        );
        let pts = |offsets: &[(f64, f64)]| -> Option<Vec<ControlPoint>> {
            offsets
                .iter()
                .map(|&(dx, dy)| {
                    GeoPoint::new(lon0 + dx, lat0 + dy)
                        .ok()
                        .map(ControlPoint::ground)
                })
                .collect()
        };
        let (entity, points) = match i % 6 {
            0 => ("140300", pts(&[(0.0, 0.0), (0.004, 0.001), (0.008, 0.0)])?),
            1 => (
                "120200",
                pts(&[(0.0, 0.0), (0.006, 0.0), (0.006, 0.004), (0.0, 0.004)])?,
            ),
            2 => ("151403", pts(&[(0.0, 0.0), (0.008, 0.003), (0.0, 0.001)])?),
            3 => (
                "170100",
                pts(&[(0.0, 0.0), (0.004, 0.003), (0.008, 0.002)])?,
            ),
            4 => ("242200", pts(&[(0.0, 0.0)])?),
            _ => ("270601", pts(&[(0.0, 0.0), (0.003, 0.002), (0.006, 0.0)])?),
        };
        let symbol = SymbolId::parse(&format!("1103250000{entity}0000")).ok()?;
        let mut d = GraphicDefinition::new(GraphicId::new(format!("g{i}")).ok()?, symbol, points);
        let numbers = ModifierValue::Numbers;
        let set = |field, value| (field, value);
        let fields = match i % 6 {
            3 => vec![
                set(ModifierField::T, ModifierValue::Text(format!("{i}"))),
                set(ModifierField::AM, numbers(vec![200.0])),
            ],
            4 => vec![
                set(ModifierField::AM, numbers(vec![100.0, 500.0])),
                set(ModifierField::AN, numbers(vec![30.0, 90.0])),
            ],
            5 => vec![],
            _ => vec![set(ModifierField::T, ModifierValue::Text(format!("{i}")))],
        };
        for (field, value) in fields {
            d.modifiers.set(field, value).ok()?;
        }
        Some(d)
    }

    fn constructions(n: usize, config: &Config) -> Vec<Construction> {
        (0..n)
            .filter_map(graphic)
            .filter_map(|d| construct(&d, config).ok())
            .collect()
    }

    fn bench(c: &mut Criterion) {
        let config = Config::default();
        let view = View::new(0, 0);
        let frame = LocalEquirectangular::new(19.99, 51.01, 50_000.0, 96.0);
        let metrics = FixedAdvanceMetrics::default();
        let mut g = c.benchmark_group("plan");
        g.sample_size(10);
        for n in SIZES {
            let defs: Vec<GraphicDefinition> = (0..n).filter_map(graphic).collect();
            g.bench_with_input(BenchmarkId::new("construct", n), &defs, |b, defs| {
                b.iter(|| {
                    defs.iter()
                        .filter_map(|d| construct(black_box(d), &config).ok())
                        .count()
                })
            });
            let built = constructions(n, &config);
            g.bench_with_input(BenchmarkId::new("render", n), &built, |b, built| {
                b.iter(|| {
                    built
                        .iter()
                        .filter_map(|c| render(black_box(c), &view, &frame, &metrics).ok())
                        .count()
                })
            });
        }
        g.finish();
    }

    criterion_group!(benches, bench);
}

#[cfg(not(target_arch = "wasm32"))]
criterion::criterion_main!(native::benches);
